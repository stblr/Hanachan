#[derive(Clone, Copy, Debug)]
pub struct Params {
    vehicle: Vehicle,
    character: Character,
}

impl Params {
    pub fn try_from_raw(vehicle_id: u8, character_id: u8) -> Option<Params> {
        Some(Params {
            vehicle: Vehicle::try_from_raw(vehicle_id)?,
            character: Character::try_from_raw(character_id)?,
        })
    }

    pub fn vehicle(&self) -> &Vehicle {
        &self.vehicle
    }

    pub fn character(&self) -> &Character {
        &self.character
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Vehicle {
    id: u8,
}

impl Vehicle {
    fn try_from_raw(id: u8) -> Option<Vehicle> {
        (id < 36).then(|| Vehicle { id })
    }

    const FILENAMES: [&'static str; 36] = [
        "sdf_kart", "mdf_kart", "ldf_kart", "sa_kart", "ma_kart", "la_kart", "sb_kart", "mb_kart",
        "lb_kart", "sc_kart", "mc_kart", "lc_kart", "sd_kart", "md_kart", "ld_kart", "se_kart",
        "me_kart", "le_kart", "sdf_bike", "mdf_bike", "ldf_bike", "sa_bike", "ma_bike", "la_bike",
        "sb_bike", "mb_bike", "lb_bike", "sc_bike", "mc_bike", "lc_bike", "sd_bike", "md_bike",
        "ld_bike", "se_bike", "me_bike", "le_bike",
    ];

    pub fn filename(&self) -> &'static str {
        Vehicle::FILENAMES[self.id as usize]
    }
}

impl From<Vehicle> for u8 {
    fn from(vehicle: Vehicle) -> u8 {
        vehicle.id
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Character {
    id: u8,
}

impl From<Character> for u8 {
    fn from(character: Character) -> u8 {
        character.id
    }
}

impl Character {
    fn try_from_raw(id: u8) -> Option<Character> {
        // TODO support Miis
        (id < 24).then(|| Character { id })
    }
}
