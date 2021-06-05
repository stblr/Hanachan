#[derive(Clone, Copy, Debug)]
pub struct Id {
    id: u8,
}

impl Id {
    pub fn try_from_raw(id: u8) -> Option<Id> {
        (id < 32).then(|| Id { id })
    }

    pub fn id(&self) -> u8 {
        self.id
    }

    pub fn filename(&self) -> &'static str {
        match self.id {
            0x00 => "castle_course",
            0x01 => "farm_course",
            0x02 => "kinoko_course",
            0x03 => "volcano_course",
            0x04 => "factory_course",
            0x05 => "shopping_course",
            0x06 => "boardcross_course",
            0x07 => "truck_course",
            0x08 => "beginner_course",
            0x09 => "senior_course",
            0x0a => "ridgehighway_course",
            0x0b => "treehouse_course",
            0x0c => "koopa_course",
            0x0d => "rainbow_course",
            0x0e => "desert_course",
            0x0f => "water_course",
            0x10 => "old_peach_gc",
            0x11 => "old_mario_gc",
            0x12 => "old_waluigi_gc",
            0x13 => "old_donkey_gc",
            0x14 => "old_falls_ds",
            0x15 => "old_desert_ds",
            0x16 => "old_garden_ds",
            0x17 => "old_town_ds",
            0x18 => "old_mario_sfc",
            0x19 => "old_obake_sfc",
            0x1a => "old_mario_64",
            0x1b => "old_sherbet_64",
            0x1c => "old_koopa_64",
            0x1d => "old_donkey_64",
            0x1e => "old_koopa_gba",
            0x1f => "old_heyho_gba",
            _ => unreachable!(),
        }
    }
}
