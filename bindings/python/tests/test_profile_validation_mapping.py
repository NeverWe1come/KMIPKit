from __future__ import annotations

from kmipkit import client


def test_profile_validation_types_expose_caller_owned_state() -> None:
    selection = client.ProfileValidationSelection(("baseline_client",))
    state = client.ProfileObjectState("object-7", original_creation_date_present=False)
    context = client.ProfileValidationContext().with_object_state(0, state)

    assert selection.profile_ids == ("baseline_client",)
    assert context.object_states == ((0, state),)


def test_client_exposes_explicit_profile_exchange_methods() -> None:
    assert callable(client.Client.execute_with_profiles)
    assert callable(client.Client.execute_with_profiles_and_options)
    assert callable(client.Client.execute_with_profiles_and_context)
    assert callable(client.Client.execute_with_profiles_and_context_and_options)
