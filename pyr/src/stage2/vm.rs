use pyr_arch::{
    addr::{IpaAddr, PhysAddr},
    page_table::{Built, MemAttr, Stage2Tables},
};

use crate::stage2::scratch;

pub struct Stage2Vm {
    tables: Stage2Tables<Built>,
}

impl Stage2Vm {
    pub fn identity_1gib() -> Self {
        let scratch = scratch::get_mut();

        let mut tables =
            Stage2Tables::new(&mut scratch.tables.root, &mut scratch.tables.l2);

        tables
            .map_range(
                IpaAddr::new(0x4000_0000),
                PhysAddr::new(0x4000_0000),
                1024_usize.pow(3),
                MemAttr::Normal,
            )
            .unwrap_or_else(|_| panic_stage2_map_failed());

        Self {
            tables: tables.build(),
        }
    }

    pub fn root_pa(&self) -> PhysAddr {
        self.tables.root_pa()
    }

    pub fn root_raw(&self) -> u64 {
        self.tables.root_raw()
    }

    pub fn enable(&self) {
        super::enable::enable_stage2(self.root_raw());
    }
}

fn panic_stage2_map_failed() -> ! {
    crate::log!("stage2: map_range failed");
    loop {
        core::hint::spin_loop();
    }
}
