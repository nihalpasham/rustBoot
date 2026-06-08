module boot_state_machine

abstract sig State {}
abstract sig BootState extends State {}
abstract sig UpdateState extends State {}

one sig BootInNew extends BootState {}
one sig BootInTesting extends BootState {}
one sig BootInSuccess extends BootState {}
one sig UpdateInNew extends UpdateState {}
one sig UpdateInUpdating extends UpdateState {}

sig BootImage {
    boot: one BootState,
    update: one UpdateState
}

fact ValidTransitions {
    // Images start with a fresh boot and new update
    some i: BootImage | i.boot = BootInNew and i.update = UpdateInNew
}

pred testImage[i, i': BootImage] {
    i.boot = BootInNew
    i'.boot = BootInTesting
    i'.update = i.update
}

pred confirmImage[i, i': BootImage] {
    i.boot = BootInTesting
    i'.boot = BootInSuccess
    i'.update = i.update
}

pred startUpdate[i, i': BootImage] {
    i.update = UpdateInNew
    i'.update = UpdateInUpdating
    i'.boot = i.boot
}

pred rollback[i, i': BootImage] {
    i.boot = BootInTesting
    i'.boot = BootInNew
    i'.update = i.update
}

assert NoInvalidTransitions {
    // It should be impossible to transition from SuccessState
    all i, i': BootImage |
        i.boot = BootInSuccess implies not (
            i'.boot != i.boot or i'.update != i.update
        )
}

check NoInvalidTransitions for 10

run { some i, i': BootImage | testImage[i, i'] } for 5
run { some i, i': BootImage | confirmImage[i, i'] } for 5
run { some i, i': BootImage | startUpdate[i, i'] } for 5