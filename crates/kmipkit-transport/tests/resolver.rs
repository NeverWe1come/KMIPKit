//! Resolver behavior tests against the real private resolver and loopback DNS.
//!
//! The Hickory 0.26.3 system-config loaders use the platform resolver API,
//! hosts file, and global search configuration. They flatten DNS servers and
//! do not preserve per-interface split-DNS routing; this is an explicit
//! platform limitation, not a fallback to an alternate endpoint.

use std::collections::BTreeMap;
use std::io::Cursor;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};
use std::str::FromStr;
use std::sync::Arc;
use std::time::{Duration, Instant};

use hickory_resolver::config::{ConnectionConfig, NameServerConfig, ResolverConfig, ResolverOpts};
use hickory_resolver::hosts::Hosts;
use hickory_resolver::proto::rr::Name;
use kmipkit_test_support::{DnsQueryType, LocalDnsFixture};
use tokio::task::JoinSet;

// Include the production module rather than a test resolver so runtime tests
// observe Hickory traffic and the production bounds.
#[allow(dead_code)]
#[path = "../src/resolver.rs"]
mod resolver;

use resolver::{ResolveFailure, Resolver, ResolverLimits};

const RESOLUTION_TIMEOUT: Duration = Duration::from_secs(12);

fn resolver_config(servers: &[SocketAddr], search: &[&str]) -> ResolverConfig {
    resolver_config_with_transport(servers, search, false)
}

fn resolver_config_with_transport(
    servers: &[SocketAddr],
    search: &[&str],
    tcp: bool,
) -> ResolverConfig {
    let mut nameservers = Vec::with_capacity(servers.len());
    for address in servers {
        let mut connection = if tcp {
            ConnectionConfig::tcp()
        } else {
            ConnectionConfig::udp()
        };
        connection.port = address.port();
        nameservers.push(NameServerConfig::new(address.ip(), true, vec![connection]));
    }

    let search_domains = search
        .iter()
        .map(|domain| Name::from_str(domain).expect("test search domain parses"))
        .collect();
    ResolverConfig::from_parts(None, search_domains, nameservers)
}

fn local_resolver(fixture: &LocalDnsFixture, search: &[&str]) -> Resolver {
    Resolver::from_config(resolver_config(&[fixture.local_addr()], search), None)
        .expect("the loopback DNS configuration builds")
}

fn fixture(records: &[(&str, Vec<IpAddr>)]) -> LocalDnsFixture {
    let records = records
        .iter()
        .map(|(name, addresses)| ((*name).to_owned(), addresses.clone()))
        .collect::<BTreeMap<_, _>>();
    LocalDnsFixture::bind(records).expect("the fixture accepts loopback records")
}

fn record_a(last_octet: u8) -> IpAddr {
    IpAddr::V4(Ipv4Addr::new(127, 0, 0, last_octet))
}

async fn wait_for_questions(
    fixture: &LocalDnsFixture,
    name: &str,
    expected_a: usize,
    expected_aaaa: usize,
) {
    tokio::time::timeout(Duration::from_secs(2), async {
        loop {
            if fixture.query_count(name, DnsQueryType::A) >= expected_a
                && fixture.query_count(name, DnsQueryType::Aaaa) >= expected_aaaa
            {
                break;
            }
            tokio::time::sleep(Duration::from_millis(2)).await;
        }
    })
    .await
    .expect("the expected DNS questions reach the loopback fixture");
}

fn query_counts_across(
    fixtures: [&LocalDnsFixture; 3],
    name: &str,
    query_type: DnsQueryType,
) -> [usize; 3] {
    fixtures.map(|fixture| fixture.query_count(name, query_type))
}

async fn wait_for_initial_fanout(
    fixtures: [&LocalDnsFixture; 3],
    name: &str,
) -> ([usize; 3], [usize; 3]) {
    tokio::time::timeout(Duration::from_secs(2), async {
        loop {
            let a_counts = query_counts_across(fixtures, name, DnsQueryType::A);
            let aaaa_counts = query_counts_across(fixtures, name, DnsQueryType::Aaaa);
            assert!(a_counts.iter().sum::<usize>() <= 2);
            assert!(aaaa_counts.iter().sum::<usize>() <= 2);
            if a_counts.iter().sum::<usize>() == 2 && aaaa_counts.iter().sum::<usize>() == 2 {
                break (a_counts, aaaa_counts);
            }
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
    })
    .await
    .expect("the A and AAAA requests reach the loopback nameservers while replies are held")
}

async fn observe_held_fanout<T>(
    fixtures: [&LocalDnsFixture; 3],
    name: &str,
    result_receiver: &mut tokio::sync::oneshot::Receiver<T>,
    observation_window: Duration,
) -> ([usize; 3], [usize; 3]) {
    let observation_started = Instant::now();
    while observation_started.elapsed() < observation_window {
        let current_counts = (
            query_counts_across(fixtures, name, DnsQueryType::A),
            query_counts_across(fixtures, name, DnsQueryType::Aaaa),
        );
        for per_question in [&current_counts.0, &current_counts.1] {
            assert!(
                per_question.iter().sum::<usize>() <= 2,
                "a question does not fan out to a third nameserver before its held requests time out"
            );
        }
        assert!(
            matches!(
                result_receiver.try_recv(),
                Err(tokio::sync::oneshot::error::TryRecvError::Empty)
            ),
            "the lookup remains pending while all DNS responses are held"
        );
        tokio::time::sleep(Duration::from_millis(5)).await;
    }

    let observed_counts = (
        query_counts_across(fixtures, name, DnsQueryType::A),
        query_counts_across(fixtures, name, DnsQueryType::Aaaa),
    );
    for per_question in [&observed_counts.0, &observed_counts.1] {
        assert!(per_question.iter().sum::<usize>() <= 2);
    }
    observed_counts
}

fn assert_resolver_limits(limits: ResolverLimits) {
    assert_eq!(limits.retries_after_initial_attempt, 1);
    assert_eq!(limits.concurrent_nameserver_requests_per_query, 2);
    assert_eq!(limits.max_active_requests_per_upstream_connection, 32);
    assert_eq!(limits.response_cache_entries_per_client, 128);
    assert_eq!(limits.max_address_candidates, 16);
}

#[tokio::test]
async fn system_resolver_initializes_from_the_platform_configuration() {
    // This runs on each supported CI OS: Hickory selects Unix resolv.conf,
    // Windows adapter/registry configuration, or Apple System Configuration.
    let resolver = Resolver::from_system_config()
        .expect("the current platform exposes a usable system DNS configuration");

    assert_resolver_limits(resolver.limits());
}

#[test]
fn resolver_records_the_system_split_dns_limitation() {
    assert!(
        resolver::SYSTEM_DNS_LIMITATIONS
            .contains("does not preserve per-interface split-DNS routing")
    );
}

#[tokio::test]
async fn injected_hosts_and_search_configuration_are_used_before_upstream_queries() {
    let dns_name = "service.search.kmipkit.test";
    let fixture = fixture(&[(dns_name, vec![record_a(21)])]);
    let config = resolver_config(&[fixture.local_addr()], &["search.kmipkit.test"]);
    let mut hosts = Hosts::default();
    hosts
        .read_hosts_conf(Cursor::new(
            b"127.0.0.22 local-only.kmipkit.test local-alias.kmipkit.test\n\
              ::1 local-only.kmipkit.test local-alias.kmipkit.test\n",
        ))
        .expect("the test hosts mapping parses");
    let resolver = Resolver::from_config(config, Some(hosts))
        .expect("the injected resolver configuration builds");

    let hosts_result = resolver
        .lookup_candidates(
            "local-only.kmipkit.test",
            Instant::now() + RESOLUTION_TIMEOUT,
        )
        .await
        .expect("hosts-file names resolve locally");
    assert!(hosts_result.contains(&record_a(22)));
    assert!(hosts_result.contains(&IpAddr::V6(Ipv6Addr::LOCALHOST)));
    assert_eq!(
        fixture.query_count("local-only.kmipkit.test", DnsQueryType::A),
        0,
        "hosts results do not issue an upstream query"
    );
    assert_eq!(
        fixture.query_count("local-only.kmipkit.test", DnsQueryType::Aaaa),
        0,
        "hosts results do not issue an upstream query for the other address family"
    );

    let searched_result = resolver
        .lookup_candidates("service", Instant::now() + RESOLUTION_TIMEOUT)
        .await
        .expect("the configured search suffix resolves the short name");
    assert_eq!(searched_result, vec![record_a(21)]);
    assert!(
        fixture.query_count(dns_name, DnsQueryType::A) > 0,
        "the configured search suffix reaches the local DNS fixture"
    );
}

#[tokio::test]
async fn one_retry_means_no_more_than_two_attempts_for_each_question() {
    let name = "retry.kmipkit.test";
    let fixture = fixture(&[(name, vec![record_a(23), IpAddr::V6(Ipv6Addr::LOCALHOST)])]);
    fixture.drop_next_questions(name, DnsQueryType::A, 1);
    fixture.drop_next_questions(name, DnsQueryType::Aaaa, 1);
    let resolver = local_resolver(&fixture, &[]);

    let resolved = resolver
        .lookup_candidates(name, Instant::now() + RESOLUTION_TIMEOUT)
        .await
        .expect("each address family succeeds after its discarded first response");

    assert!(resolved.contains(&record_a(23)));
    assert!(resolved.contains(&IpAddr::V6(Ipv6Addr::LOCALHOST)));
    assert_eq!(fixture.query_count(name, DnsQueryType::A), 2);
    assert_eq!(fixture.query_count(name, DnsQueryType::Aaaa), 2);
}

#[tokio::test]
async fn one_lookup_never_has_more_than_two_concurrent_nameserver_requests() {
    let name = "parallel.kmipkit.test";
    let records = [(name, vec![record_a(24), IpAddr::V6(Ipv6Addr::LOCALHOST)])];
    let first = fixture(&records);
    let second = fixture(&records);
    let third = fixture(&records);
    first.hold_responses();
    second.hold_responses();
    third.hold_responses();
    let resolver = Arc::new(
        Resolver::from_config(
            resolver_config(
                &[first.local_addr(), second.local_addr(), third.local_addr()],
                &[],
            ),
            None,
        )
        .expect("three local nameservers build"),
    );
    assert_resolver_limits(resolver.limits());
    let lookup_resolver = Arc::clone(&resolver);
    let lookup_name = name.to_owned();
    let (result_sender, mut result_receiver) = tokio::sync::oneshot::channel();
    let lookup = tokio::spawn(async move {
        let result = lookup_resolver
            .lookup_candidates(&lookup_name, Instant::now() + RESOLUTION_TIMEOUT)
            .await;
        let _send_result = result_sender.send(result);
    });

    wait_for_initial_fanout([&first, &second, &third], name).await;

    // Hickory's default request timeout bounds the initial parallel nameserver
    // round. Observe almost that entire interval, with the responses still held,
    // so a timer-delayed third nameserver request cannot hide behind a 100 ms
    // snapshot. The margin avoids crossing into the permitted next round after
    // the first requests have timed out.
    let observation_window = ResolverOpts::default()
        .timeout
        .saturating_sub(Duration::from_millis(250));
    assert!(!observation_window.is_zero());
    let observed_counts = observe_held_fanout(
        [&first, &second, &third],
        name,
        &mut result_receiver,
        observation_window,
    )
    .await;
    for per_nameserver in [&observed_counts.0[..], &observed_counts.1[..]] {
        assert!(
            per_nameserver.iter().any(|count| *count == 0),
            "at least one of three nameservers remains unqueried while responses are held"
        );
    }
    assert!(first.peak_active_responses() > 0);
    assert!(second.peak_active_responses() > 0);
    assert!(third.peak_active_responses() > 0);
    first.release_responses();
    second.release_responses();
    third.release_responses();
    lookup.await.expect("the resolver task completes");
    result_receiver
        .await
        .expect("the lookup result is delivered")
        .expect("the released DNS responses resolve");
}

#[tokio::test]
async fn active_request_limit_applies_per_upstream_connection_not_per_client() {
    let first_name = "upstream-a.kmipkit.test";
    let second_name = "upstream-b.kmipkit.test";
    let first = fixture(&[(first_name, vec![record_a(25)])]);
    let second = fixture(&[(second_name, vec![record_a(26)])]);
    first.hold_responses();
    second.hold_responses();
    let resolver = Arc::new(
        Resolver::from_config(
            resolver_config_with_transport(&[first.local_addr(), second.local_addr()], &[], true),
            None,
        )
        .expect("two local upstreams build"),
    );
    let mut lookups = JoinSet::new();
    let sentinel_resolver = Arc::clone(&resolver);
    let (sentinel_sender, mut sentinel_receiver) = tokio::sync::oneshot::channel();
    lookups.spawn(async move {
        let result = sentinel_resolver
            .lookup_candidates(
                "held-cap-sentinel.kmipkit.test",
                Instant::now() + RESOLUTION_TIMEOUT,
            )
            .await;
        let _send_result = sentinel_sender.send(result);
    });
    for index in 0..47 {
        let current = Arc::clone(&resolver);
        let hostname = format!("host-{index}.kmipkit.test");
        lookups.spawn(async move {
            current
                .lookup_candidates(&hostname, Instant::now() + RESOLUTION_TIMEOUT)
                .await
        });
    }

    let stable_counts = tokio::time::timeout(Duration::from_secs(3), async {
        let mut stable_since = None;
        loop {
            let active = [first.active_tcp_requests(), second.active_tcp_requests()];
            let peak = [
                first.peak_active_tcp_requests_per_connection(),
                second.peak_active_tcp_requests_per_connection(),
            ];
            assert!(
                active.iter().all(|count| *count <= 32) && peak.iter().all(|count| *count <= 32),
                "no upstream connection exceeds 32 held requests, including transient peaks"
            );
            assert!(
                matches!(
                    sentinel_receiver.try_recv(),
                    Err(tokio::sync::oneshot::error::TryRecvError::Empty)
                ),
                "a designated lookup remains pending while the cap is saturated"
            );
            if active == [32, 32] && peak == [32, 32] {
                let since = stable_since.get_or_insert_with(Instant::now);
                if since.elapsed() >= Duration::from_millis(500) {
                    break active;
                }
            } else {
                stable_since = None;
            }
            tokio::time::sleep(Duration::from_millis(2)).await;
        }
    })
    .await
    .expect("each independent TCP upstream holds exactly 32 requests stably with work pending");
    assert_eq!(stable_counts, [32, 32]);
    assert_eq!(first.active_tcp_requests(), 32);
    assert_eq!(second.active_tcp_requests(), 32);
    assert_eq!(first.peak_active_tcp_requests_per_connection(), 32);
    assert_eq!(second.peak_active_tcp_requests_per_connection(), 32);
    assert!(
        first.active_tcp_requests() + second.active_tcp_requests() > 32,
        "the per-upstream cap is not an aggregate client cap"
    );

    first.release_responses();
    second.release_responses();
    while let Some(result) = lookups.join_next().await {
        result.expect("lookup tasks complete");
    }
    let _sentinel_result = sentinel_receiver
        .await
        .expect("the released sentinel lookup result is delivered");
}

#[tokio::test]
async fn response_cache_is_per_resolver_and_capped_at_128_answers() {
    let names = (0..66)
        .map(|index| {
            (
                format!("cache-{index}.kmipkit.test"),
                vec![
                    record_a(u8::try_from(index + 1).expect("record index fits")),
                    IpAddr::V6(Ipv6Addr::LOCALHOST),
                ],
            )
        })
        .collect::<BTreeMap<_, _>>();
    let fixture = LocalDnsFixture::bind(names).expect("cache test records are valid loopback data");
    let host_names = [
        "cache-63.kmipkit.test",
        "cache-64.kmipkit.test",
        "cache-65.kmipkit.test",
    ];
    let host_addresses = [record_a(100), record_a(101), record_a(102)];
    let hosts_text = host_names
        .iter()
        .zip(host_addresses)
        .map(|(name, address)| format!("{address} {name}\n"))
        .collect::<String>();
    let mut hosts = Hosts::default();
    hosts
        .read_hosts_conf(Cursor::new(hosts_text.into_bytes()))
        .expect("A-only hosts entries parse");
    let resolver =
        Resolver::from_config(resolver_config(&[fixture.local_addr()], &[]), Some(hosts))
            .expect("the cache resolver uses its injected A-only hosts entries");
    let initial_names = (0..63)
        .map(|index| format!("cache-{index}.kmipkit.test"))
        .collect::<Vec<_>>();
    let probe_names = &host_names[..2];

    for name in &initial_names {
        let candidates = resolver
            .lookup_candidates(name, Instant::now() + RESOLUTION_TIMEOUT)
            .await
            .expect("each known A+AAAA answer resolves");
        assert!(candidates.iter().any(IpAddr::is_ipv4));
        assert!(candidates.iter().any(IpAddr::is_ipv6));
        assert_eq!(fixture.query_count(name, DnsQueryType::A), 1);
        assert_eq!(fixture.query_count(name, DnsQueryType::Aaaa), 1);
    }
    for (name, address) in probe_names.iter().zip(host_addresses) {
        let candidates = resolver
            .lookup_candidates(name, Instant::now() + RESOLUTION_TIMEOUT)
            .await
            .expect("the A hosts result and known AAAA answer resolve");
        assert!(candidates.contains(&address));
        assert!(candidates.contains(&IpAddr::V6(Ipv6Addr::LOCALHOST)));
        assert_eq!(fixture.query_count(name, DnsQueryType::A), 0);
        assert_eq!(fixture.query_count(name, DnsQueryType::Aaaa), 1);
    }

    // 63 dual-stack lookups plus two AAAA-only DNS answers fill exactly 128
    // response-cache entries. Repeating all keys proves they are still hits.
    for name in &initial_names {
        resolver
            .lookup_candidates(name, Instant::now() + RESOLUTION_TIMEOUT)
            .await
            .expect("all 126 cached address-family answers remain available");
        assert_eq!(fixture.query_count(name, DnsQueryType::A), 1);
        assert_eq!(fixture.query_count(name, DnsQueryType::Aaaa), 1);
    }
    for (name, address) in probe_names.iter().zip(host_addresses) {
        let candidates = resolver
            .lookup_candidates(name, Instant::now() + RESOLUTION_TIMEOUT)
            .await
            .expect("both cached AAAA answers remain available at the exact limit");
        assert!(candidates.contains(&address));
        assert_eq!(fixture.query_count(name, DnsQueryType::Aaaa), 1);
    }

    let first_name = initial_names[0].as_str();
    let first_a_before = fixture.query_count(first_name, DnsQueryType::A);
    let first_aaaa_before = fixture.query_count(first_name, DnsQueryType::Aaaa);
    local_resolver(&fixture, &[])
        .lookup_candidates(first_name, Instant::now() + RESOLUTION_TIMEOUT)
        .await
        .expect("a second resolver client has an independent cache");
    assert_eq!(
        fixture.query_count(first_name, DnsQueryType::A),
        first_a_before + 1
    );
    assert_eq!(
        fixture.query_count(first_name, DnsQueryType::Aaaa),
        first_aaaa_before + 1
    );

    resolver
        .lookup_candidates(host_names[2], Instant::now() + RESOLUTION_TIMEOUT)
        .await
        .expect("the 129th distinct DNS response is obtained for the third probe");
    assert_eq!(fixture.query_count(host_names[2], DnsQueryType::A), 0);
    assert_eq!(fixture.query_count(host_names[2], DnsQueryType::Aaaa), 1);

    let mut before_probe = initial_names
        .iter()
        .flat_map(|name| {
            [
                (
                    name.clone(),
                    DnsQueryType::A,
                    fixture.query_count(name, DnsQueryType::A),
                ),
                (
                    name.clone(),
                    DnsQueryType::Aaaa,
                    fixture.query_count(name, DnsQueryType::Aaaa),
                ),
            ]
        })
        .collect::<Vec<_>>();
    before_probe.extend(probe_names.iter().map(|name| {
        (
            (*name).to_owned(),
            DnsQueryType::Aaaa,
            fixture.query_count(name, DnsQueryType::Aaaa),
        )
    }));
    for name in &initial_names {
        resolver
            .lookup_candidates(name, Instant::now() + RESOLUTION_TIMEOUT)
            .await
            .expect("known answers remain resolvable after cache overflow");
    }
    for (name, address) in probe_names.iter().zip(host_addresses) {
        let candidates = resolver
            .lookup_candidates(name, Instant::now() + RESOLUTION_TIMEOUT)
            .await
            .expect("host and DNS answers remain resolvable after cache overflow");
        assert!(candidates.contains(&address));
    }
    let additional_queries = before_probe
        .iter()
        .map(|(name, query_type, before)| {
            fixture
                .query_count(name, *query_type)
                .saturating_sub(*before)
        })
        .sum::<usize>();
    assert!(
        additional_queries > 0,
        "the 129th cached response evicts at least one of the first 128 entries"
    );
}

#[tokio::test]
async fn candidates_are_ordered_and_capped_across_address_families() {
    let name = "ordered.kmipkit.test";
    let addresses = (1..=16).rev().map(record_a).collect::<Vec<_>>();
    let mut answers = addresses.clone();
    answers.push(IpAddr::V6(Ipv6Addr::LOCALHOST));
    let fixture = fixture(&[(name, answers)]);
    let resolver = local_resolver(&fixture, &[]);

    let candidates = resolver
        .lookup_candidates(name, Instant::now() + RESOLUTION_TIMEOUT)
        .await
        .expect("the loopback record set resolves");

    assert_eq!(candidates.len(), 16);
    assert_eq!(candidates[0], IpAddr::V6(Ipv6Addr::LOCALHOST));
    assert_eq!(candidates[1..], addresses[..15]);
}

#[tokio::test]
async fn timeout_and_cancellation_stop_the_caller_visible_lookup() {
    let name = "blocked.kmipkit.test";
    let fixture = fixture(&[(name, vec![record_a(28)])]);
    fixture.hold_responses();
    let resolver = Arc::new(local_resolver(&fixture, &[]));
    let timed_out = resolver
        .lookup_candidates(name, Instant::now() + Duration::from_millis(150))
        .await
        .expect_err("the blocked lookup reaches its caller deadline");
    assert_eq!(timed_out.failure(), ResolveFailure::Timeout);
    assert!(fixture.query_count(name, DnsQueryType::A) > 0);

    let cancellation_name = "cancelled.kmipkit.test";
    let cancellation_resolver = Arc::clone(&resolver);
    let lookup = tokio::spawn(async move {
        cancellation_resolver
            .lookup_candidates(cancellation_name, Instant::now() + RESOLUTION_TIMEOUT)
            .await
    });
    wait_for_questions(&fixture, "cancelled.kmipkit.test", 1, 1).await;
    lookup.abort();
    assert!(
        lookup
            .await
            .expect_err("the lookup task is cancelled")
            .is_cancelled()
    );
    fixture.release_responses();
    tokio::time::sleep(Duration::from_millis(100)).await;
    assert_eq!(
        fixture.query_count("cancelled.kmipkit.test", DnsQueryType::A),
        1,
        "cancelled lookup work does not retry or dispatch later"
    );
}

#[tokio::test]
async fn nxdomain_is_reported_without_exposing_the_hostname() {
    let name = "sensitive-name.kmipkit.test";
    let fixture = fixture(&[]);
    fixture.set_nxdomain(name);
    let resolver = local_resolver(&fixture, &[]);

    let error = resolver
        .lookup_candidates(name, Instant::now() + RESOLUTION_TIMEOUT)
        .await
        .expect_err("NXDOMAIN is an error for endpoint resolution");

    assert_eq!(error.failure(), ResolveFailure::NxDomain);
    assert!(!error.to_string().contains(name));
}

#[tokio::test]
async fn lookup_returns_no_candidates_before_the_loopback_dns_response_arrives() {
    let name = "no-request-before-dns.kmipkit.test";
    let fixture = fixture(&[(name, vec![record_a(29)])]);
    fixture.hold_responses();
    let resolver = Arc::new(local_resolver(&fixture, &[]));
    let resolving = Arc::clone(&resolver);
    let (resolved_sender, mut resolved_receiver) = tokio::sync::oneshot::channel();
    let lookup = tokio::spawn(async move {
        let result = resolving
            .lookup_candidates(name, Instant::now() + RESOLUTION_TIMEOUT)
            .await;
        let _send_result = resolved_sender.send(result);
    });

    wait_for_questions(&fixture, name, 1, 1).await;
    assert!(
        matches!(
            resolved_receiver.try_recv(),
            Err(tokio::sync::oneshot::error::TryRecvError::Empty)
        ),
        "the same lookup future cannot return candidates while DNS responses are held"
    );
    // Request-byte ordering is asserted by the raw-TLS and HTTPS adapter tests;
    // this resolver test proves the caller has no candidate before DNS completes.
    fixture.release_responses();
    lookup.await.expect("the resolver task completes");
    let candidates = resolved_receiver
        .await
        .expect("the lookup result is delivered")
        .expect("the released DNS answer resolves");
    assert_eq!(candidates, vec![record_a(29)]);
}
