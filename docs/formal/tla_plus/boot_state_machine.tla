-------------------------- MODULE boot_state_machine --------------------------
EXTENDS Naturals, FiniteSets

CONSTANTS
    BootInNewState,
    BootInTestingState,
    BootInSuccessState,
    UpdateInNewState,
    UpdateInUpdatingState

VARIABLES bootState, updateState

vars == <<bootState, updateState>>

TypeOK ==
    /\ bootState \in {BootInNewState, BootInTestingState, BootInSuccessState}
    /\ updateState \in {UpdateInNewState, UpdateInUpdatingState}

Init ==
    /\ bootState = BootInNewState
    /\ updateState = UpdateInNewState

TestImage ==
    /\ bootState = BootInNewState
    /\ bootState' = BootInTestingState
    /\ updateState' = updateState

ConfirmImage ==
    /\ bootState = BootInTestingState
    /\ bootState' = BootInSuccessState
    /\ updateState' = updateState

StartUpdate ==
    /\ updateState = UpdateInNewState
    /\ updateState' = UpdateInUpdatingState
    /\ bootState' = bootState

Rollback ==
    /\ bootState = BootInTestingState
    /\ bootState' = BootInNewState
    /\ updateState' = updateState

Next ==
    \/ TestImage
    \/ ConfirmImage
    \/ StartUpdate
    \/ Rollback

Spec == Init /\ [][Next]_vars

Invariant ==
    \/ bootState = BootInNewState
    \/ bootState = BootInTestingState
    \/ bootState = BootInSuccessState

=============================================================================