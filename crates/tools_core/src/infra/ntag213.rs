#[cfg(test)]
mod tests {
    use nfc1::{BaudRate, Modulation, ModulationType};

    #[test]
    fn test() {
        let mut ctx = nfc1::Context::new().unwrap();
        let rs = ctx.list_devices(5).unwrap();
        println!("Found readers: {:?}", rs);
        let mut device = ctx.open().unwrap();
        device.initiator_init().unwrap();
        let modulation = Modulation {
            modulation_type: ModulationType::Iso14443a,
            baud_rate: BaudRate::Baud106,
        };
        let rs = device.initiator_list_passive_targets(&modulation, 5).unwrap();
        println!("Selected target: {:?}", rs);
    }
}