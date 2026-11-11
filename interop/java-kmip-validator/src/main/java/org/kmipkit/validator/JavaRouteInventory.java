package org.kmipkit.validator;

import org.kmipkit.extensions.ClientBatchItem;

import java.lang.reflect.Method;
import java.lang.reflect.Modifier;
import java.util.LinkedHashMap;
import java.util.Map;
import java.util.TreeSet;

/** Explicit inventory of Java public request routes for every operation in the shared catalog. */
public final class JavaRouteInventory {
    private static final Map<String, Route> ROUTES = createRoutes();

    private JavaRouteInventory() { }

    /** Returns a route classification for every operation found in the catalog. */
    public static Map<String, Route> classify(CatalogLoader.Catalog catalog) {
        var operations = new TreeSet<String>();
        catalog.cases().forEach(testCase -> {
            if (!"server_exchange".equals(testCase.execution().mode())) return;
            Route route = routeFor(testCase);
            if (route.status() == Status.UNCLASSIFIED) {
                throw new IllegalArgumentException("Catalog operation has no Java route classification: "
                        + testCase.operation());
            }
            operations.add(testCase.operation());
        });
        var classified = new LinkedHashMap<String, Route>();
        for (String operation : operations) {
            Route route = ROUTES.get(operation);
            if (route == null) {
                throw new IllegalArgumentException("Catalog operation has no Java route classification: " + operation);
            }
            classified.put(operation, route);
        }
        return Map.copyOf(classified);
    }

    /** Returns a route status for one operation, using an explicit fallback for future catalog additions. */
    public static Route routeFor(String operation) {
        return ROUTES.getOrDefault(operation, new Route(operation, Status.UNCLASSIFIED, "No Java route classification exists."));
    }

    /** Classifies one case while preserving its client-to-server or server-to-client workflow. */
    public static Route routeFor(CatalogLoader.CatalogCase testCase) {
        if ("local_model".equals(testCase.execution().mode())) {
            return new Route(testCase.operation(), Status.LOCAL_MODEL,
                    "This case is checked locally through public TTLV or protocol-value APIs.");
        }
        String workflow = testCase.requires().path("workflow_type").asText();
        if ("server_to_client".equals(workflow)) {
            return new Route(testCase.operation(), Status.WORKFLOW_UNAVAILABLE,
                    "This catalog case requires a server-initiated workflow; the Java consumer has no server request listener.");
        }
        return routeFor(testCase.operation());
    }

    private static Map<String, Route> createRoutes() {
        var routes = new LinkedHashMap<String, Route>();
        executable(routes, "Discover Versions", "discoverVersions");
        executable(routes, "Ping", "ping");
        executable(routes, "Query", "query");
        executable(routes, "Create", "create");
        mutatingExecutable(routes, "Log", "log");
        pendingRecipe(routes, "Get Constraints", "getConstraints");
        pendingRecipe(routes, "Get Attribute List", "getAttributeList");

        typed(routes, "Activate", "activate");
        typed(routes, "Add Attribute", "addAttribute");
        typed(routes, "Adjust Attribute", "adjustAttribute");
        typed(routes, "Archive", "archive");
        payloadOnly(routes, "Cancel", "CancelRequest");
        typed(routes, "Certify", "certify");
        typed(routes, "Check", "check");
        typed(routes, "Create Key Pair", "createKeyPair");
        typed(routes, "Create Split Key", "createSplitKey");
        typed(routes, "Decrypt", "decrypt");
        typed(routes, "Delegated Login", "delegatedLogin");
        typed(routes, "Delete Attribute", "deleteAttribute");
        typed(routes, "Derive Key", "deriveKey");
        typed(routes, "Destroy", "destroy");
        typed(routes, "Encrypt", "encrypt");
        typed(routes, "Export", "export");
        typed(routes, "Get", "get");
        typed(routes, "Get Attributes", "getAttributes");
        typed(routes, "Get Usage Allocation", "getUsageAllocation");
        typed(routes, "Hash", "hash");
        typed(routes, "Import", "importObject");
        typed(routes, "Interop", "interop");
        typed(routes, "Join Split Key", "joinSplitKey");
        typed(routes, "Locate", "locate");
        typed(routes, "Login", "login");
        typed(routes, "Logout", "logout");
        typed(routes, "MAC", "mac");
        typed(routes, "MAC Verify", "macVerify");
        typed(routes, "Modify Attribute", "modifyAttribute");
        noPublicRoute(routes, "Notify", "Notify is a server-initiated operation outside the client request API.");
        typed(routes, "Obtain Lease", "obtainLease");
        typed(routes, "PKCS#11", "pkcs11");
        payloadOnly(routes, "Poll", "PollRequest");
        payloadOnly(routes, "Process", "ProcessRequest");
        noPublicRoute(routes, "Put", "The pinned Java package exposes no public Put request constructor.");
        payloadOnly(routes, "Query Asynchronous Requests", "QueryAsyncRequestsRequest");
        typed(routes, "Re-Provision", "reprovision");
        typed(routes, "Re-certify", "recertify");
        typed(routes, "Re-key", "rekey");
        typed(routes, "Re-key Key Pair", "rekeyKeyPair");
        typed(routes, "Recover", "recover");
        typed(routes, "Register", "register");
        typed(routes, "Revoke", "revoke");
        typed(routes, "RNG Retrieve", "rngRetrieve");
        typed(routes, "RNG Seed", "rngSeed");
        typed(routes, "Set Attribute", "setAttribute");
        typed(routes, "Set Constraints", "setConstraints");
        typed(routes, "Set Defaults", "setDefaults");
        noPublicRoute(routes, "Set Endpoint Role", "The pinned Java package exposes no public Set Endpoint Role request constructor.");
        typed(routes, "Sign", "sign");
        typed(routes, "Signature Verify", "signatureVerify");
        typed(routes, "Validate", "validate");
        return Map.copyOf(routes);
    }

    private static void executable(Map<String, Route> routes, String operation, String method) {
        requirePublicFactory(method);
        routes.put(operation, new Route(operation, Status.EXECUTABLE_TYPED,
                "JavaKmipDriver sends this typed request and verifies its typed response."));
    }

    private static void mutatingExecutable(Map<String, Route> routes, String operation, String method) {
        requirePublicFactory(method);
        routes.put(operation, new Route(operation, Status.EXECUTABLE_TYPED,
                "ClientBatchItem." + method + " appends a message to the server log; mutation opt-in is required."));
    }

    private static void typed(Map<String, Route> routes, String operation, String method) {
        requirePublicFactory(method);
        routes.put(operation, new Route(operation, Status.EXECUTABLE_TYPED,
                "JavaKmipDriver constructs ClientBatchItem." + method
                        + " and checks the installed typed response reader; catalog preconditions remain observable."));
    }

    private static void pendingRecipe(Map<String, Route> routes, String operation, String method) {
        requirePublicFactory(method);
        routes.put(operation, new Route(operation, Status.EXECUTABLE_TYPED,
                "JavaKmipDriver constructs ClientBatchItem." + method
                        + " and checks its installed typed response reader; server-side object preconditions are reported."));
    }

    private static void payloadOnly(Map<String, Route> routes, String operation, String requestType) {
        routes.put(operation, new Route(operation, Status.PAYLOAD_ONLY,
                "The Java package exposes " + requestType
                        + " payload construction, but not a ClientBatchItem route for sending it."));
    }

    private static void noPublicRoute(Map<String, Route> routes, String operation, String reason) {
        routes.put(operation, new Route(operation, Status.NO_PUBLIC_ROUTE, reason));
    }

    private static void requirePublicFactory(String methodName) {
        for (Method method : ClientBatchItem.class.getMethods()) {
            if (method.getName().equals(methodName) && Modifier.isStatic(method.getModifiers())) return;
        }
        throw new IllegalStateException("Pinned Java package is missing expected public request factory: " + methodName);
    }

    /** Request construction and transmission state visible to the independent consumer. */
    public enum Status {
        EXECUTABLE_TYPED,
        TYPED_API_PENDING,
        PAYLOAD_ONLY,
        NO_PUBLIC_ROUTE,
        WORKFLOW_UNAVAILABLE,
        LOCAL_MODEL,
        UNCLASSIFIED
    }

    /** Auditable route status and a user-safe reason for its current coverage state. */
    public record Route(String operation, Status status, String reason) { }
}
