use soundfont_rs as sf2;
use std::rc::Rc;

use std::io::{Read, Seek, SeekFrom};
use std::path::PathBuf;

use crate::gen::{self, Gen};
use crate::modulator::Mod;
use crate::soundfont::Sample;
use crate::synth::Synth;
use crate::voice::FluidVoiceAddMod;
use std::path::Path;

const GEN_SET: GenFlags = 1;
const GEN_VELOCITY: u32 = 47;
const GEN_KEYNUM: u32 = 46;
const FLUID_VOICE_ADD: FluidVoiceAddMod = 1;
const GEN_OVERRIDEROOTKEY: GenType = 58;
const GEN_EXCLUSIVECLASS: GenType = 57;
const GEN_SAMPLEMODE: GenType = 54;
const GEN_ENDLOOPADDRCOARSEOFS: GenType = 50;
const GEN_STARTLOOPADDRCOARSEOFS: GenType = 45;
const GEN_ENDADDRCOARSEOFS: GenType = 12;
const GEN_STARTADDRCOARSEOFS: GenType = 4;
const GEN_ENDLOOPADDROFS: GenType = 3;
const GEN_STARTLOOPADDROFS: GenType = 2;
const GEN_ENDADDROFS: GenType = 1;
const GEN_STARTADDROFS: GenType = 0;
const GEN_LAST: GenType = 60;
const FLUID_VOICE_OVERWRITE: FluidVoiceAddMod = 0;
type GenType = u32;
type GenFlags = u32;

pub(super) struct DefaultSoundFont {
    pub(super) filename: PathBuf,
    pub(super) sampledata: Rc<Vec<i16>>,
    sample: Vec<Rc<Sample>>,
    pub(super) preset: Vec<Rc<DefaultPreset>>,
}

impl DefaultSoundFont {
    fn get_sample(&mut self, name: &str) -> Option<Rc<Sample>> {
        self.sample
            .iter()
            .find(|sample| name == &sample.name)
            .map(|s| s.clone())
    }

    pub(super) fn load(path: &Path) -> Result<Self, ()> {
        let filename = path.to_owned();
        let mut file = std::fs::File::open(&filename).unwrap();

        let data = sf2::data::SFData::load(&mut file);
        let mut sf2 = sf2::SoundFont2::from_data(data);
        sf2.sort_presets();

        let smpl = sf2.sample_data.smpl.as_ref().unwrap();

        let samplepos = smpl.offset() + 8;
        let samplesize = smpl.len() as usize;

        let sampledata = Rc::new(Self::load_sampledata(&mut file, samplepos, samplesize)?);

        let mut defsfont = DefaultSoundFont {
            filename,
            sample: Vec::new(),
            sampledata,
            preset: Vec::new(),
        };

        for sfsample in sf2.sample_headers.iter() {
            let sample = Sample::import_sfont(sfsample, &mut defsfont)?;
            let mut sample = sample;

            unsafe {
                sample.optimize_sample();
            }

            defsfont.sample.push(Rc::new(sample));
        }

        for sfpreset in sf2.presets.iter() {
            let preset = unsafe { DefaultPreset::import_sfont(&sf2, sfpreset, &mut defsfont)? };
            defsfont.preset.push(Rc::new(preset));
        }

        Ok(defsfont)
    }

    fn load_sampledata(
        file: &mut std::fs::File,
        sample_pos: u64,
        sample_size: usize,
    ) -> Result<Vec<i16>, ()> {
        if file.seek(SeekFrom::Start(sample_pos)).is_err() {
            log::error!("Failed to seek position in data file",);
            return Err(());
        }

        let mut sample_data = vec![0u8; sample_size];
        if file.read_exact(&mut sample_data).is_err() {
            log::error!("Failed to read sample data");
            return Err(());
        }

        let sample_data: Vec<i16> = sample_data
            .chunks(2)
            .map(|num| {
                if num.len() == 2 {
                    i16::from_le_bytes([num[0], num[1]])
                } else {
                    log::error!("Wrong sample data");
                    0
                }
            })
            .collect();

        Ok(sample_data)
    }
}

pub(super) struct DefaultPreset {
    pub(super) name: String,
    pub(super) bank: u32,
    pub(super) num: u32,
    global_zone: Option<PresetZone>,
    zones: Vec<PresetZone>,
}

impl DefaultPreset {
    unsafe fn import_sfont(
        sf2: &sf2::SoundFont2,
        sfpreset: &sf2::Preset,
        sfont: &mut DefaultSoundFont,
    ) -> Result<Self, ()> {
        let mut preset = DefaultPreset {
            name: String::new(),
            bank: 0 as i32 as u32,
            num: 0 as i32 as u32,
            global_zone: None,
            zones: Vec::new(),
        };

        if sfpreset.header.name.len() != 0 {
            preset.name = sfpreset.header.name.clone();
        } else {
            preset.name = format!(
                "Bank:{},Preset{}",
                sfpreset.header.bank, sfpreset.header.preset
            );
        }

        preset.bank = sfpreset.header.bank as u32;
        preset.num = sfpreset.header.preset as u32;

        for (id, sfzone) in sfpreset.zones.iter().enumerate() {
            let name = format!("{}/{}", sfpreset.header.name, id);
            let zone = PresetZone::import_sfont(name, sf2, sfzone, sfont)?;

            if id == 0 && zone.inst.is_none() {
                preset.global_zone = Some(zone);
            } else {
                preset.zones.push(zone);
            }
        }

        Ok(preset)
    }
}

struct PresetZone {
    #[allow(dead_code)]
    name: String,
    inst: Option<Instrument>,
    keylo: u8,
    keyhi: u8,
    vello: i32,
    velhi: i32,
    gen: [Gen; 60],
    mods: Vec<Mod>,
}

impl PresetZone {
    fn import_sfont(
        name: String,
        sf2: &sf2::SoundFont2,
        sfzone: &sf2::Zone,
        sfont: &mut DefaultSoundFont,
    ) -> Result<Self, ()> {
        let mut zone = Self {
            name,
            inst: None,
            keylo: 0,
            keyhi: 128,
            vello: 0 as i32,
            velhi: 128 as i32,
            gen: gen::get_default_values(),
            mods: Vec::new(),
        };

        for sfgen in sfzone
            .gen_list
            .iter()
            .filter(|g| g.ty != sf2::data::SFGeneratorType::Instrument)
        {
            match sfgen.ty {
                sf2::data::SFGeneratorType::KeyRange | sf2::data::SFGeneratorType::VelRange => {
                    let amount = sfgen.amount.as_range().unwrap();
                    zone.keylo = amount.low;
                    zone.keyhi = amount.high;
                }
                _ => {
                    // FIXME: some generators have an unsigned word amount value but i don't know which ones
                    zone.gen[sfgen.ty as usize].val = *sfgen.amount.as_i16().unwrap() as f64;
                    zone.gen[sfgen.ty as usize].flags = GEN_SET as u8;
                }
            }
        }
        if let Some(id) = sfzone.instrument() {
            let inst = Instrument::import_sfont(sf2, &sf2.instruments[*id as usize], sfont)?;

            zone.inst = Some(inst);
        }
        // Import the modulators (only SF2.1 and higher)
        for mod_src in sfzone.mod_list.iter() {
            let mod_dest = Mod::from(mod_src);

            /* Store the new modulator in the zone The order of modulators
             * will make a difference, at least in an instrument context: The
             * second modulator overwrites the first one, if they only differ
             * in amount. */
            zone.mods.push(mod_dest);
        }

        Ok(zone)
    }
}

#[derive(Clone)]
struct Instrument {
    // [u8;21]
    name: String,
    global_zone: Option<InstrumentZone>,
    zones: Vec<InstrumentZone>,
}

impl Instrument {
    fn import_sfont(
        sf2: &sf2::SoundFont2,
        new_inst: &sf2::Instrument,
        sfont: &mut DefaultSoundFont,
    ) -> Result<Self, ()> {
        let mut inst = Self {
            name: String::new(),
            global_zone: None,
            zones: Vec::new(),
        };

        if new_inst.header.name.len() > 0 {
            inst.name = new_inst.header.name.clone();
        } else {
            inst.name = "<untitled>".into();
        }
        for (id, new_zone) in new_inst.zones.iter().enumerate() {
            let name = format!("{}/{}", new_inst.header.name, id);
            let zone = InstrumentZone::import_sfont(name, sf2, new_zone, &mut *sfont)?;
            if id == 0 && zone.sample.is_none() {
                inst.global_zone = Some(zone);
            } else {
                inst.zones.push(zone);
            }
        }

        Ok(inst)
    }
}

#[derive(Clone)]
#[repr(C)]
struct InstrumentZone {
    name: String,
    sample: Option<Rc<Sample>>,
    keylo: u8,
    keyhi: u8,
    vello: i32,
    velhi: i32,
    gen: [Gen; 60],
    mods: Vec<Mod>,
}

impl InstrumentZone {
    fn import_sfont(
        name: String,
        sf2: &sf2::SoundFont2,
        new_zone: &sf2::Zone,
        sfont: &mut DefaultSoundFont,
    ) -> Result<InstrumentZone, ()> {
        let mut keylo = 0;
        let mut keyhi = 128;
        let mut gen = gen::get_default_values();

        for new_gen in new_zone
            .gen_list
            .iter()
            .filter(|g| g.ty != sf2::data::SFGeneratorType::SampleID)
        {
            match new_gen.ty {
                sf2::data::SFGeneratorType::KeyRange | sf2::data::SFGeneratorType::VelRange => {
                    let amount = new_gen.amount.as_range().unwrap();
                    keylo = amount.low;
                    keyhi = amount.high;
                }
                _ => {
                    // FIXME: some generators have an unsigned word amount value but i don't know which ones
                    gen[new_gen.ty as usize].val = *new_gen.amount.as_i16().unwrap() as f64;
                    gen[new_gen.ty as usize].flags = GEN_SET as u8;
                }
            }
        }

        let sample = if let Some(sample_id) = new_zone.sample() {
            let sample = sf2.sample_headers.get(*sample_id as usize).unwrap();
            let sample = sfont.get_sample(&sample.name);
            if sample.is_none() {
                log::error!("Couldn't find sample name",);
                return Err(());
            }
            sample
        } else {
            None
        };

        let mut mods = Vec::new();

        for new_mod in new_zone.mod_list.iter() {
            let mod_dest = Mod::from(new_mod);
            /* Store the new modulator in the zone
             * The order of modulators will make a difference, at least in an instrument context:
             * The second modulator overwrites the first one, if they only differ in amount. */
            mods.push(mod_dest);
        }

        Ok(InstrumentZone {
            name: name,
            sample,
            keylo,
            keyhi,
            vello: 0,
            velhi: 128,
            gen,
            mods,
        })
    }
}

impl Synth {
    /// noteon
    pub(crate) fn sf_noteon(&mut self, chan: u8, key: u8, vel: i32) -> Result<(), ()> {
        fn preset_zone_inside_range(zone: &PresetZone, key: u8, vel: i32) -> bool {
            zone.keylo <= key && zone.keyhi >= key && zone.vello <= vel && zone.velhi >= vel
        }

        fn inst_zone_inside_range(zone: &InstrumentZone, key: u8, vel: i32) -> bool {
            zone.keylo <= key && zone.keyhi >= key && zone.vello <= vel && zone.velhi >= vel
        }

        fn sample_in_rom(sample: &Sample) -> i32 {
            // sampletype & FLUID_SAMPLETYPE_ROM
            sample.sampletype & 0x8000
        }

        unsafe {
            let preset = {
                let preset = self.channel[chan as usize].preset.as_ref().unwrap();
                preset.data.clone()
            };

            let mut mod_list: [*const Mod; 64] = [0 as *const Mod; 64]; // list for 'sorting' preset modulators

            let mut global_preset_zone = &preset.global_zone;

            // run thru all the zones of this preset
            for preset_zone in preset.zones.iter() {
                // check if the note falls into the key and velocity range of this preset
                if preset_zone_inside_range(preset_zone, key, vel) {
                    let inst = preset_zone.inst.as_ref().unwrap();

                    let mut global_inst_zone = &inst.global_zone;

                    // run thru all the zones of this instrument
                    for inst_zone in inst.zones.iter() {
                        // make sure this instrument zone has a valid sample
                        let sample = &inst_zone.sample;
                        if !(sample.is_none() || sample_in_rom(&sample.as_ref().unwrap()) != 0) {
                            // check if the note falls into the key and velocity range of this instrument
                            if inst_zone_inside_range(inst_zone, key, vel) && !sample.is_none() {
                                // this is a good zone. allocate a new synthesis process and initialize it
                                let voice_id = self.alloc_voice(
                                    sample.as_ref().unwrap().clone(),
                                    chan,
                                    key,
                                    vel,
                                );

                                if let Some(voice_id) = voice_id {
                                    // Instrument level, generators
                                    let mut i = 0;
                                    while i < GEN_LAST as i32 {
                                        /* SF 2.01 section 9.4 'bullet' 4:
                                         *
                                         * A generator in a local instrument zone supersedes a
                                         * global instrument zone generator.  Both cases supersede
                                         * the default generator -> voice_gen_set */
                                        if inst_zone.gen[i as usize].flags != 0 {
                                            self.voices[voice_id.0]
                                                .gen_set(i, inst_zone.gen[i as usize].val);
                                        } else if let Some(global_inst_zone) = &global_inst_zone {
                                            if global_inst_zone.gen[i as usize].flags as i32 != 0 {
                                                self.voices[voice_id.0].gen_set(
                                                    i,
                                                    global_inst_zone.gen[i as usize].val,
                                                );
                                            }
                                        } else {
                                            /* The generator has not been defined in this instrument.
                                             * Do nothing, leave it at the default.
                                             */
                                        }
                                        i += 1
                                    }

                                    /* global instrument zone, modulators: Put them all into a
                                     * list. */
                                    let mut mod_list_count = 0;
                                    if let Some(global_inst_zone) = &mut global_inst_zone {
                                        for m in global_inst_zone.mods.iter() {
                                            mod_list[mod_list_count] = m;
                                            mod_list_count += 1;
                                        }
                                    }

                                    /* local instrument zone, modulators.
                                     * Replace modulators with the same definition in the list:
                                     * SF 2.01 page 69, 'bullet' 8
                                     */
                                    for m in inst_zone.mods.iter() {
                                        /* 'Identical' modulators will be deleted by setting their
                                         *  list entry to NULL.  The list length is known, NULL
                                         *  entries will be ignored later.  SF2.01 section 9.5.1
                                         *  page 69, 'bullet' 3 defines 'identical'.  */
                                        let mut i = 0;
                                        while i < mod_list_count {
                                            if !mod_list[i].is_null()
                                                && m.test_identity(
                                                    mod_list[i as usize].as_ref().unwrap(),
                                                )
                                            {
                                                mod_list[i] = 0 as *mut Mod
                                            }
                                            i += 1
                                        }

                                        /* Finally add the new modulator to to the list. */
                                        mod_list[mod_list_count] = m;

                                        mod_list_count += 1;
                                    }

                                    // Add instrument modulators (global / local) to the voice.
                                    let mut i = 0;
                                    while i < mod_list_count {
                                        let mod_0 = mod_list[i as usize];
                                        if !mod_0.is_null() {
                                            // disabled modulators CANNOT be skipped.

                                            /* Instrument modulators -supersede- existing (default)
                                             * modulators.  SF 2.01 page 69, 'bullet' 6 */
                                            self.voices[voice_id.0].add_mod(
                                                mod_0.as_ref().unwrap(),
                                                FLUID_VOICE_OVERWRITE,
                                            );
                                        }
                                        i += 1
                                    }

                                    /* Preset level, generators */
                                    let mut i = 0;
                                    while i < GEN_LAST {
                                        /* SF 2.01 section 8.5 page 58: If some generators are
                                         * encountered at preset level, they should be ignored */
                                        if i != GEN_STARTADDROFS
                                            && i != GEN_ENDADDROFS
                                            && i != GEN_STARTLOOPADDROFS
                                            && i != GEN_ENDLOOPADDROFS
                                            && i != GEN_STARTADDRCOARSEOFS
                                            && i != GEN_ENDADDRCOARSEOFS
                                            && i != GEN_STARTLOOPADDRCOARSEOFS
                                            && i != GEN_KEYNUM
                                            && i != GEN_VELOCITY
                                            && i != GEN_ENDLOOPADDRCOARSEOFS
                                            && i != GEN_SAMPLEMODE
                                            && i != GEN_EXCLUSIVECLASS
                                            && i != GEN_OVERRIDEROOTKEY
                                        {
                                            /* SF 2.01 section 9.4 'bullet' 9: A generator in a
                                             * local preset zone supersedes a global preset zone
                                             * generator.  The effect is -added- to the destination
                                             * summing node -> voice_gen_incr */
                                            if preset_zone.gen[i as usize].flags != 0 {
                                                self.voices[voice_id.0]
                                                    .gen_incr(i, preset_zone.gen[i as usize].val);
                                            } else if let Some(global_preset_zone) =
                                                &global_preset_zone
                                            {
                                                if global_preset_zone.gen[i as usize].flags != 0 {
                                                    self.voices[voice_id.0].gen_incr(
                                                        i,
                                                        global_preset_zone.gen[i as usize].val,
                                                    );
                                                }
                                            } else {
                                                /* The generator has not been defined in this preset
                                                 * Do nothing, leave it unchanged.
                                                 */
                                            }
                                        } /* if available at preset level */
                                        i += 1
                                    } /* for all generators */

                                    /* Global preset zone, modulators: put them all into a
                                     * list. */
                                    let mut mod_list_count = 0;
                                    if let Some(global_preset_zone) = &mut global_preset_zone {
                                        for m in global_preset_zone.mods.iter() {
                                            mod_list[mod_list_count] = m;
                                            mod_list_count += 1;
                                        }
                                    }

                                    /* Process the modulators of the local preset zone.  Kick
                                     * out all identical modulators from the global preset zone
                                     * (SF 2.01 page 69, second-last bullet) */
                                    for m in preset_zone.mods.iter() {
                                        let mut i = 0;
                                        while i < mod_list_count {
                                            if !mod_list[i].is_null()
                                                && m.test_identity(
                                                    mod_list[i as usize].as_ref().unwrap(),
                                                )
                                            {
                                                mod_list[i] = 0 as *mut Mod
                                            }
                                            i += 1
                                        }

                                        /* Finally add the new modulator to the list. */
                                        mod_list[mod_list_count] = m;

                                        mod_list_count += 1;
                                    }

                                    // Add preset modulators (global / local) to the voice.
                                    let mut i = 0;
                                    while i < mod_list_count {
                                        let m = mod_list[i];
                                        if !m.is_null() && (*m).amount != 0.0 {
                                            // disabled modulators can be skipped.

                                            /* Preset modulators -add- to existing instrument /
                                             * default modulators.  SF2.01 page 70 first bullet on
                                             * page */
                                            self.voices[voice_id.0]
                                                .add_mod(m.as_ref().unwrap(), FLUID_VOICE_ADD);
                                        }
                                        i += 1
                                    }

                                    // add the synthesis process to the synthesis loop.
                                    self.start_voice(voice_id);

                                    /* Store the ID of the first voice that was created by this noteon event.
                                     * Exclusive class may only terminate older voices.
                                     * That avoids killing voices, which have just been created.
                                     * (a noteon event can create several voice processes with the same exclusive
                                     * class - for example when using stereo samples)
                                     */
                                } else {
                                    return Err(());
                                }
                            }
                        }
                    }
                }
            }

            Ok(())
        }
    }
}
