//! Pin Mux and Pin Control types

/// MUX Mode Select Field
pub enum MuxMode {
    Alt0,
    Alt1,
    Alt2,
    Alt3,
    Alt4,
    Alt5,
    Alt6,
    Alt7,
}

/// Software Input On Field.
pub enum Sion {
    Enabled,
    Disabled,
}

/// Drive Strength Field
pub enum Dse {
    DseX1,
    DseX2,
    DseX6,
    Unimplemented,
}
/// Slew Rate Field
pub enum Fsel {
    Slow,
    Fast,
}
/// Open Drain Enable Field
pub enum Ode {
    Enabled,
    Disabled,
}
/// Control IO ports PS
pub enum Pue {
    PullUp,
    PullDown,
}
/// Hysteresis Enable Field
pub enum Hys {
    Enabled,
    Disabled,
}
/// Pull Resistors Enable Field
pub enum Pe {
    Enabled,
    Disabled,
}
