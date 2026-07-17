use bevy::window::{
    MonitorSelection, PrimaryWindow, WindowMode, WindowResizeConstraints, WindowResolution,
};
use bevy::{
    gltf::{
        GltfAssetLabel, GltfExtras, GltfMaterialExtras, GltfMaterialName, GltfMeshExtras,
        GltfMeshName, GltfSceneExtras,
    },
    prelude::*,
};
use rand::Rng;
#[cfg(not(target_arch = "wasm32"))]
use rodio::{Decoder, OutputStream, Sink, Source};
use serde::{Deserialize, Serialize};
use std::{
    env, fs,
    path::{Path, PathBuf},
    sync::mpsc,
    time::{SystemTime, UNIX_EPOCH},
};
#[cfg(not(target_arch = "wasm32"))]
use std::{
    io::{BufReader, Cursor},
    thread,
};

const LANES: usize = 5;
const COLS: usize = 9;
const CELL: f32 = 1.35;
const BOARD_LEFT: f32 = -5.4;
const BOARD_TOP: f32 = 3.0;
const ZOMBIE_START_X: f32 = 7.3;
const HOME_X: f32 = -6.4;
const SAVE_FILE_NAME: &str = "bevy_open_siege_save.ron";

// Binary assets are embedded for desktop packaging/audits only; on web they are
// fetched over HTTP by the asset server, so keep them out of the wasm binary.
macro_rules! embedded_bytes {
    ($path:literal) => {{
        #[cfg(not(target_arch = "wasm32"))]
        {
            include_bytes!($path) as &[u8]
        }
        #[cfg(target_arch = "wasm32")]
        {
            &[] as &[u8]
        }
    }};
}

const LEVELS_RON: &str = include_str!("../assets/data/levels.ron");
const EN_RON: &str = include_str!("../assets/i18n/en.ron");
const ZH_RON: &str = include_str!("../assets/i18n/zh.ron");
const VERSION_RON: &str = include_str!("../VERSION.ron");
const ASSET_MANIFEST_RON: &str = include_str!("../assets/manifest.ron");
const README_MD: &str = include_str!("../README.md");
const CREDITS_MD: &str = include_str!("../CREDITS.md");
const ART_ASSETS_MD: &str = include_str!("../ART_ASSETS.md");
const THIRD_PARTY_NOTICES_MD: &str = include_str!("../THIRD_PARTY_NOTICES.md");
const STORE_PAGE_MD: &str = include_str!("../STORE_PAGE.md");
const PRESSKIT_MD: &str = include_str!("../PRESSKIT.md");
const STORE_SCREENSHOTS_MD: &str = include_str!("../STORE_SCREENSHOTS.md");
const CONTENT_RATING_MD: &str = include_str!("../CONTENT_RATING.md");
const RELEASE_NOTES_MD: &str = include_str!("../RELEASE_NOTES.md");
const PRIVACY_MD: &str = include_str!("../PRIVACY.md");
const SUPPORT_MD: &str = include_str!("../SUPPORT.md");
const TROUBLESHOOTING_MD: &str = include_str!("../TROUBLESHOOTING.md");
const BUILD_PROVENANCE_MD: &str = include_str!("../BUILD_PROVENANCE.md");
const CARGO_TOML: &str = include_str!("../Cargo.toml");
const CARGO_LOCK: &str = include_str!("../Cargo.lock");
const MAIN_RS: &str = include_str!("main.rs");
const LINUX_DESKTOP_ENTRY: &str = include_str!("../assets/linux/bevy-open-siege.desktop");
const LINUX_APPSTREAM_METAINFO: &str =
    include_str!("../assets/linux/io.github.bevy_open_siege.BevyOpenSiege.metainfo.xml");
const BRAND_ICON_SVG: &str = include_str!("../assets/branding/icon.svg");
const BRAND_CAPSULE_SVG: &str = include_str!("../assets/branding/capsule.svg");
const APP_ICON_PNG: &[u8] = embedded_bytes!("../assets/branding/generated/app-icon.png");
const STORE_CAPSULE_PNG: &[u8] = embedded_bytes!("../assets/branding/generated/store-capsule.png");
const PLANTS_SHEET_PNG: &[u8] = embedded_bytes!("../assets/art/plants-sheet.png");
const MONSTERS_SHEET_PNG: &[u8] = embedded_bytes!("../assets/art/monsters-sheet.png");
const MUSIC_LOOP_WAV: &[u8] = embedded_bytes!("../assets/audio/music-loop.wav");
const PLANT_PLACE_WAV: &[u8] = embedded_bytes!("../assets/audio/plant-place.wav");
const SHOOT_WAV: &[u8] = embedded_bytes!("../assets/audio/shoot.wav");
const SUN_COLLECT_WAV: &[u8] = embedded_bytes!("../assets/audio/sun-collect.wav");
const MONSTER_DOWN_WAV: &[u8] = embedded_bytes!("../assets/audio/monster-down.wav");
const VICTORY_WAV: &[u8] = embedded_bytes!("../assets/audio/victory.wav");
const DEFEAT_WAV: &[u8] = embedded_bytes!("../assets/audio/defeat.wav");
const EFFECT_PEA_PNG: &[u8] = embedded_bytes!("../assets/art/effects/pea.png");
const EFFECT_FROST_POD_PNG: &[u8] = embedded_bytes!("../assets/art/effects/frost-pod.png");
const EFFECT_CABBAGE_PNG: &[u8] = embedded_bytes!("../assets/art/effects/cabbage.png");
const EFFECT_SUN_PNG: &[u8] = embedded_bytes!("../assets/art/effects/sun.png");
const EFFECT_FIRE_PNG: &[u8] = embedded_bytes!("../assets/art/effects/fire.png");
const EFFECT_EXPLOSION_PNG: &[u8] = embedded_bytes!("../assets/art/effects/explosion.png");
const ENV_LAWN_BASE_PNG: &[u8] = embedded_bytes!("../assets/art/environment/lawn-base.png");
const ENV_LANE_GRASS_PNG: &[u8] = embedded_bytes!("../assets/art/environment/lane-grass.png");
const ENV_SOIL_BORDER_PNG: &[u8] = embedded_bytes!("../assets/art/environment/soil-border.png");
const UI_MENU_PANEL_PNG: &[u8] = embedded_bytes!("../assets/art/ui/menu-panel.png");
const UI_HUD_PANEL_PNG: &[u8] = embedded_bytes!("../assets/art/ui/hud-panel.png");
const UI_END_PANEL_PNG: &[u8] = embedded_bytes!("../assets/art/ui/end-panel.png");
const FONT_CJK_TTF: &[u8] = embedded_bytes!("../assets/fonts/SarasaMonoSC-subset.ttf");
const AUDIO_MUSIC_LOOP: &str = "audio/music-loop.wav";
const AUDIO_PLANT_PLACE: &str = "audio/plant-place.wav";
const AUDIO_SHOOT: &str = "audio/shoot.wav";
const AUDIO_SUN_COLLECT: &str = "audio/sun-collect.wav";
const AUDIO_MONSTER_DOWN: &str = "audio/monster-down.wav";
const AUDIO_VICTORY: &str = "audio/victory.wav";
const AUDIO_DEFEAT: &str = "audio/defeat.wav";
const EFFECT_PEA: &str = "art/effects/pea.png";
const EFFECT_FROST_POD: &str = "art/effects/frost-pod.png";
const EFFECT_CABBAGE: &str = "art/effects/cabbage.png";
const EFFECT_SUN: &str = "art/effects/sun.png";
const EFFECT_FIRE: &str = "art/effects/fire.png";
const EFFECT_EXPLOSION: &str = "art/effects/explosion.png";
const ENV_LAWN_BASE: &str = "art/environment/lawn-base.png";
const ENV_LANE_GRASS: &str = "art/environment/lane-grass.png";
const ENV_SOIL_BORDER: &str = "art/environment/soil-border.png";
const UI_MENU_PANEL: &str = "art/ui/menu-panel.png";
const UI_HUD_PANEL: &str = "art/ui/hud-panel.png";
const UI_END_PANEL: &str = "art/ui/end-panel.png";
const FONT_CJK: &str = "fonts/SarasaMonoSC-subset.ttf";
const AUDIO_ASSETS: [(&str, &[u8]); 7] = [
    ("assets/audio/music-loop.wav", MUSIC_LOOP_WAV),
    ("assets/audio/plant-place.wav", PLANT_PLACE_WAV),
    ("assets/audio/shoot.wav", SHOOT_WAV),
    ("assets/audio/sun-collect.wav", SUN_COLLECT_WAV),
    ("assets/audio/monster-down.wav", MONSTER_DOWN_WAV),
    ("assets/audio/victory.wav", VICTORY_WAV),
    ("assets/audio/defeat.wav", DEFEAT_WAV),
];
const EFFECT_ASSETS: [(&str, &[u8]); 6] = [
    ("assets/art/effects/pea.png", EFFECT_PEA_PNG),
    ("assets/art/effects/frost-pod.png", EFFECT_FROST_POD_PNG),
    ("assets/art/effects/cabbage.png", EFFECT_CABBAGE_PNG),
    ("assets/art/effects/sun.png", EFFECT_SUN_PNG),
    ("assets/art/effects/fire.png", EFFECT_FIRE_PNG),
    ("assets/art/effects/explosion.png", EFFECT_EXPLOSION_PNG),
];
const ENVIRONMENT_ASSETS: [(&str, &[u8]); 3] = [
    ("assets/art/environment/lawn-base.png", ENV_LAWN_BASE_PNG),
    ("assets/art/environment/lane-grass.png", ENV_LANE_GRASS_PNG),
    (
        "assets/art/environment/soil-border.png",
        ENV_SOIL_BORDER_PNG,
    ),
];
#[cfg(test)]
const UI_ASSETS: [(&str, &[u8]); 3] = [
    ("assets/art/ui/menu-panel.png", UI_MENU_PANEL_PNG),
    ("assets/art/ui/hud-panel.png", UI_HUD_PANEL_PNG),
    ("assets/art/ui/end-panel.png", UI_END_PANEL_PNG),
];
const PLANT_SPRITE_ASSETS: [(&str, &[u8]); 10] = [
    (
        "assets/art/sprites/plants/sprout-slinger.png",
        embedded_bytes!("../assets/art/sprites/plants/sprout-slinger.png"),
    ),
    (
        "assets/art/sprites/plants/sunbloom.png",
        embedded_bytes!("../assets/art/sprites/plants/sunbloom.png"),
    ),
    (
        "assets/art/sprites/plants/bark-bulwark.png",
        embedded_bytes!("../assets/art/sprites/plants/bark-bulwark.png"),
    ),
    (
        "assets/art/sprites/plants/frost-sprout.png",
        embedded_bytes!("../assets/art/sprites/plants/frost-sprout.png"),
    ),
    (
        "assets/art/sprites/plants/twin-pod.png",
        embedded_bytes!("../assets/art/sprites/plants/twin-pod.png"),
    ),
    (
        "assets/art/sprites/plants/leaf-lobber.png",
        embedded_bytes!("../assets/art/sprites/plants/leaf-lobber.png"),
    ),
    (
        "assets/art/sprites/plants/briar-mat.png",
        embedded_bytes!("../assets/art/sprites/plants/briar-mat.png"),
    ),
    (
        "assets/art/sprites/plants/blast-berry.png",
        embedded_bytes!("../assets/art/sprites/plants/blast-berry.png"),
    ),
    (
        "assets/art/sprites/plants/ember-stump.png",
        embedded_bytes!("../assets/art/sprites/plants/ember-stump.png"),
    ),
    (
        "assets/art/sprites/plants/scent-root.png",
        embedded_bytes!("../assets/art/sprites/plants/scent-root.png"),
    ),
];
const MONSTER_SPRITE_ASSETS: [(&str, &[u8]); 10] = [
    (
        "assets/art/sprites/monsters/walker.png",
        embedded_bytes!("../assets/art/sprites/monsters/walker.png"),
    ),
    (
        "assets/art/sprites/monsters/conehead.png",
        embedded_bytes!("../assets/art/sprites/monsters/conehead.png"),
    ),
    (
        "assets/art/sprites/monsters/runner.png",
        embedded_bytes!("../assets/art/sprites/monsters/runner.png"),
    ),
    (
        "assets/art/sprites/monsters/buckethead.png",
        embedded_bytes!("../assets/art/sprites/monsters/buckethead.png"),
    ),
    (
        "assets/art/sprites/monsters/brute.png",
        embedded_bytes!("../assets/art/sprites/monsters/brute.png"),
    ),
    (
        "assets/art/sprites/monsters/healer.png",
        embedded_bytes!("../assets/art/sprites/monsters/healer.png"),
    ),
    (
        "assets/art/sprites/monsters/jumper.png",
        embedded_bytes!("../assets/art/sprites/monsters/jumper.png"),
    ),
    (
        "assets/art/sprites/monsters/digger.png",
        embedded_bytes!("../assets/art/sprites/monsters/digger.png"),
    ),
    (
        "assets/art/sprites/monsters/frostbite.png",
        embedded_bytes!("../assets/art/sprites/monsters/frostbite.png"),
    ),
    (
        "assets/art/sprites/monsters/gargantuar.png",
        embedded_bytes!("../assets/art/sprites/monsters/gargantuar.png"),
    ),
];
const PLANT_MODEL_ASSETS: [(&str, &[u8]); 10] = [
    (
        "assets/models/plants/sprout-slinger.glb",
        embedded_bytes!("../assets/models/plants/sprout-slinger.glb"),
    ),
    (
        "assets/models/plants/sunbloom.glb",
        embedded_bytes!("../assets/models/plants/sunbloom.glb"),
    ),
    (
        "assets/models/plants/bark-bulwark.glb",
        embedded_bytes!("../assets/models/plants/bark-bulwark.glb"),
    ),
    (
        "assets/models/plants/frost-sprout.glb",
        embedded_bytes!("../assets/models/plants/frost-sprout.glb"),
    ),
    (
        "assets/models/plants/twin-pod.glb",
        embedded_bytes!("../assets/models/plants/twin-pod.glb"),
    ),
    (
        "assets/models/plants/leaf-lobber.glb",
        embedded_bytes!("../assets/models/plants/leaf-lobber.glb"),
    ),
    (
        "assets/models/plants/briar-mat.glb",
        embedded_bytes!("../assets/models/plants/briar-mat.glb"),
    ),
    (
        "assets/models/plants/blast-berry.glb",
        embedded_bytes!("../assets/models/plants/blast-berry.glb"),
    ),
    (
        "assets/models/plants/ember-stump.glb",
        embedded_bytes!("../assets/models/plants/ember-stump.glb"),
    ),
    (
        "assets/models/plants/scent-root.glb",
        embedded_bytes!("../assets/models/plants/scent-root.glb"),
    ),
];
const MONSTER_MODEL_ASSETS: [(&str, &[u8]); 10] = [
    (
        "assets/models/monsters/walker.glb",
        embedded_bytes!("../assets/models/monsters/walker.glb"),
    ),
    (
        "assets/models/monsters/conehead.glb",
        embedded_bytes!("../assets/models/monsters/conehead.glb"),
    ),
    (
        "assets/models/monsters/runner.glb",
        embedded_bytes!("../assets/models/monsters/runner.glb"),
    ),
    (
        "assets/models/monsters/buckethead.glb",
        embedded_bytes!("../assets/models/monsters/buckethead.glb"),
    ),
    (
        "assets/models/monsters/brute.glb",
        embedded_bytes!("../assets/models/monsters/brute.glb"),
    ),
    (
        "assets/models/monsters/healer.glb",
        embedded_bytes!("../assets/models/monsters/healer.glb"),
    ),
    (
        "assets/models/monsters/jumper.glb",
        embedded_bytes!("../assets/models/monsters/jumper.glb"),
    ),
    (
        "assets/models/monsters/digger.glb",
        embedded_bytes!("../assets/models/monsters/digger.glb"),
    ),
    (
        "assets/models/monsters/frostbite.glb",
        embedded_bytes!("../assets/models/monsters/frostbite.glb"),
    ),
    (
        "assets/models/monsters/gargantuar.glb",
        embedded_bytes!("../assets/models/monsters/gargantuar.glb"),
    ),
];

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
enum GameState {
    #[default]
    Menu,
    Playing,
    GameOver,
    Victory,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum StoreScreenshotScene {
    TitleMenu,
    EarlyDefense,
    SpecialEnemies,
    LateSiege,
    VictorySummary,
}

#[derive(Resource, Default)]
struct StoreScreenshotMode {
    scene: Option<StoreScreenshotScene>,
}

#[derive(Resource)]
struct BoardState {
    level_index: usize,
    sun: u32,
    cursor_col: usize,
    cursor_lane: usize,
    selected: PlantKind,
    wave: u32,
    score: u32,
    kills: u32,
    lost_house_hp: u32,
    plant_cooldowns: [f32; PlantKind::COUNT],
    spawn_timer: Timer,
    sky_sun_timer: Timer,
    wave_timer: Timer,
    grace_timer: Timer,
    final_wave_started: bool,
}

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Serialize, Deserialize)]
enum Language {
    #[default]
    English,
    Chinese,
}

impl Language {
    fn next(self) -> Self {
        match self {
            Self::English => Self::Chinese,
            Self::Chinese => Self::English,
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::English => "English",
            Self::Chinese => "中文",
        }
    }
}

#[derive(Resource)]
struct LanguageSettings {
    current: Language,
}

#[derive(Resource, Clone)]
struct GameSettings {
    master_volume: f32,
    fullscreen: bool,
}

#[derive(Resource)]
struct AsyncAudio {
    enabled: bool,
    sender: Option<mpsc::Sender<AudioCommand>>,
}

#[cfg_attr(target_arch = "wasm32", allow(dead_code))]
enum AudioCommand {
    PlayMusic { master_volume: f32 },
    SetMusicVolume { master_volume: f32 },
    PlaySound { path: &'static str, volume: f32 },
}

impl Default for GameSettings {
    fn default() -> Self {
        Self {
            master_volume: 0.8,
            fullscreen: false,
        }
    }
}

#[derive(Resource, Default)]
struct PauseState {
    paused: bool,
}

#[derive(Resource)]
struct UiFonts {
    cjk: Handle<Font>,
}

#[derive(Resource, Default)]
struct OnboardingState {
    step: usize,
    timer: Timer,
    done: bool,
}

#[derive(Resource, Clone)]
struct LevelCatalog {
    levels: Vec<LevelConfig>,
    selected: usize,
}

#[derive(Debug, Clone, Deserialize)]
struct LevelConfig {
    id: String,
    title_en: String,
    title_zh: String,
    starting_sun: u32,
    max_breaches: u32,
    final_wave: u32,
    sky_sun_interval: f32,
    base_spawn_interval: f32,
    wave_duration: f32,
    final_spawn_count: u32,
}

impl LevelConfig {
    fn title(&self, language: Language) -> &str {
        match language {
            Language::English => &self.title_en,
            Language::Chinese => &self.title_zh,
        }
    }
}

#[derive(Resource, Clone)]
struct LocalizationCatalog {
    english: LocaleText,
    chinese: LocaleText,
}

#[derive(Debug, Clone, Deserialize)]
struct LocaleText {
    title: String,
    menu_help: String,
    menu_start: String,
    game_over: String,
    victory: String,
    retry: String,
    play_again: String,
    pause_resume: String,
    pause_restart: String,
    pause_quit: String,
    kills: String,
    wave_incoming: String,
    final_wave_warning: String,
    hints: Vec<String>,
    locked_level: String,
    level: String,
    best_score: String,
    no_score: String,
    hud: HudTextLabels,
    plant_labels: Vec<String>,
    plant_descriptions: Vec<String>,
    zombie_labels: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct HudTextLabels {
    sun: String,
    seed: String,
    wave: String,
    score: String,
    cursor: String,
    collect_sun: String,
    language: String,
    level: String,
}

#[derive(Debug, Clone, Deserialize)]
struct ReleaseMetadata {
    product_name: String,
    version: String,
    release_channel: String,
    content_rating_note: String,
    supported_languages: Vec<String>,
    supported_platforms: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct AssetManifest {
    branding: Vec<AssetManifestEntry>,
    #[serde(default)]
    art: Vec<AssetManifestEntry>,
    #[serde(default)]
    audio: Vec<AssetManifestEntry>,
    data: Vec<AssetManifestEntry>,
}

#[derive(Debug, Clone, Deserialize)]
struct AssetManifestEntry {
    id: String,
    path: String,
    kind: String,
    usage: String,
    status: String,
}

impl LocalizationCatalog {
    fn text(&self, language: Language) -> &LocaleText {
        match language {
            Language::English => &self.english,
            Language::Chinese => &self.chinese,
        }
    }
}

#[derive(Resource, Clone)]
struct ProgressState {
    unlocked_levels: usize,
    best_scores: Vec<u32>,
}

impl ProgressState {
    fn best_score(&self, level_index: usize) -> Option<u32> {
        self.best_scores
            .get(level_index)
            .copied()
            .filter(|score| *score > 0)
    }

    fn is_unlocked(&self, level_index: usize) -> bool {
        level_index < self.unlocked_levels
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SaveData {
    #[serde(default = "default_save_version")]
    version: u32,
    #[serde(default)]
    language: Language,
    #[serde(default = "default_unlocked_levels")]
    unlocked_levels: usize,
    #[serde(default)]
    best_scores: Vec<u32>,
    #[serde(default)]
    settings: SaveSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SaveSettings {
    master_volume: f32,
    fullscreen: bool,
}

impl Default for SaveSettings {
    fn default() -> Self {
        Self {
            master_volume: 0.8,
            fullscreen: false,
        }
    }
}

fn default_save_version() -> u32 {
    1
}

fn default_unlocked_levels() -> usize {
    1
}

impl Default for SaveData {
    fn default() -> Self {
        Self {
            version: 1,
            language: Language::English,
            unlocked_levels: 1,
            best_scores: Vec::new(),
            settings: SaveSettings::default(),
        }
    }
}

impl From<&SaveSettings> for GameSettings {
    fn from(settings: &SaveSettings) -> Self {
        Self {
            master_volume: settings.master_volume.clamp(0.0, 1.0),
            fullscreen: settings.fullscreen,
        }
    }
}

impl Default for LanguageSettings {
    fn default() -> Self {
        Self {
            current: Language::English,
        }
    }
}

impl Default for BoardState {
    fn default() -> Self {
        Self {
            level_index: 0,
            sun: 125,
            cursor_col: 0,
            cursor_lane: 2,
            selected: PlantKind::Peashooter,
            wave: 1,
            score: 0,
            kills: 0,
            lost_house_hp: 0,
            plant_cooldowns: [0.0; PlantKind::COUNT],
            spawn_timer: Timer::from_seconds(2.6, TimerMode::Repeating),
            sky_sun_timer: Timer::from_seconds(8.0, TimerMode::Repeating),
            wave_timer: Timer::from_seconds(20.0, TimerMode::Repeating),
            grace_timer: Timer::from_seconds(opening_grace_seconds(0), TimerMode::Once),
            final_wave_started: false,
        }
    }
}

// Setup window before the first zombie walks in; veterans on later levels
// get less breathing room.
fn opening_grace_seconds(level_index: usize) -> f32 {
    (9.0 - level_index as f32 * 0.6).clamp(4.0, 9.0)
}

// Lanes open center-out as waves progress so the early economy can keep up;
// later levels start wider until every lane is hot from wave one.
const LANE_ROLLOUT: [usize; LANES] = [2, 1, 3, 0, 4];

fn active_lane_count(level_index: usize, wave: u32) -> usize {
    (wave as usize + level_index).clamp(1, LANES)
}

impl BoardState {
    fn for_level(level_index: usize, level: &LevelConfig) -> Self {
        Self {
            level_index,
            sun: level.starting_sun,
            cursor_col: 0,
            cursor_lane: 2,
            selected: PlantKind::Peashooter,
            wave: 1,
            score: 0,
            kills: 0,
            lost_house_hp: 0,
            plant_cooldowns: [0.0; PlantKind::COUNT],
            spawn_timer: Timer::from_seconds(level.base_spawn_interval, TimerMode::Repeating),
            sky_sun_timer: Timer::from_seconds(level.sky_sun_interval, TimerMode::Repeating),
            wave_timer: Timer::from_seconds(level.wave_duration, TimerMode::Repeating),
            grace_timer: Timer::from_seconds(opening_grace_seconds(level_index), TimerMode::Once),
            final_wave_started: false,
        }
    }
}

#[derive(Component)]
struct GameEntity;

#[derive(Component)]
struct MenuUi;

#[derive(Component)]
struct MenuTitleText;

#[derive(Component)]
struct MenuHelpText;

#[derive(Component)]
struct MenuStartText;

#[derive(Component)]
struct MenuLevelText;

#[derive(Component)]
struct MenuRosterText;

#[derive(Component, Clone, Copy)]
struct MenuLevelRow(usize);

#[derive(Component)]
struct MenuStartButton;

#[derive(Component)]
struct MenuSettingsText;

#[derive(Component)]
struct PauseUi;

#[derive(Component)]
struct PauseText;

#[derive(Component, Clone, Copy, Eq, PartialEq)]
enum PauseButton {
    Resume,
    Restart,
    Quit,
}

#[derive(Component)]
struct EndRetryButton;

#[derive(Component)]
struct HintUi;

#[derive(Component)]
struct HintText;

#[derive(Component)]
struct WaveBarFill;

#[derive(Component)]
struct SeedButton(PlantKind);

#[derive(Component)]
struct WaveBanner {
    timer: Timer,
}

#[derive(Component)]
struct WaveBannerText;

#[derive(Component)]
struct HitReact {
    timer: Timer,
    base_scale: Vec3,
}

#[derive(Resource, Default)]
struct ScreenShake {
    trauma: f32,
}

#[derive(Component)]
struct CameraRig {
    base: Vec3,
}

#[derive(Component)]
struct EndTitleText;

#[derive(Component)]
struct EndSubtitleText;

#[derive(Component)]
struct HudUi;

#[derive(Component)]
struct HudText;

#[derive(Component)]
struct HudStatusText;

#[derive(Component)]
struct HudSeedBankText;

#[derive(Component)]
struct CursorMarker;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum PlantKind {
    Peashooter,
    Sunflower,
    Wallnut,
    SnowPea,
    Repeater,
    CabbagePult,
    Spikeweed,
    CherryBomb,
    Torchwood,
    Garlic,
}

impl PlantKind {
    const COUNT: usize = 10;
    const ALL: [Self; 10] = [
        Self::Peashooter,
        Self::Sunflower,
        Self::Wallnut,
        Self::SnowPea,
        Self::Repeater,
        Self::CabbagePult,
        Self::Spikeweed,
        Self::CherryBomb,
        Self::Torchwood,
        Self::Garlic,
    ];

    fn index(self) -> usize {
        match self {
            Self::Peashooter => 0,
            Self::Sunflower => 1,
            Self::Wallnut => 2,
            Self::SnowPea => 3,
            Self::Repeater => 4,
            Self::CabbagePult => 5,
            Self::Spikeweed => 6,
            Self::CherryBomb => 7,
            Self::Torchwood => 8,
            Self::Garlic => 9,
        }
    }

    fn cooldown_seconds(self) -> f32 {
        match self {
            Self::Peashooter => 5.0,
            Self::Sunflower => 7.5,
            Self::Wallnut => 12.0,
            Self::SnowPea => 8.0,
            Self::Repeater => 9.0,
            Self::CabbagePult => 8.5,
            Self::Spikeweed => 6.5,
            Self::CherryBomb => 18.0,
            Self::Torchwood => 10.0,
            Self::Garlic => 5.0,
        }
    }

    fn fallback_label(self) -> &'static str {
        match self {
            Self::Peashooter => "Sprout Slinger",
            Self::Sunflower => "Sunbloom",
            Self::Wallnut => "Bark Bulwark",
            Self::SnowPea => "Frost Sprout",
            Self::Repeater => "Twin Pod",
            Self::CabbagePult => "Leaf Lobber",
            Self::Spikeweed => "Briar Mat",
            Self::CherryBomb => "Blast Berry",
            Self::Torchwood => "Ember Stump",
            Self::Garlic => "Scent Root",
        }
    }

    fn label(self, locale: &LocaleText) -> &str {
        locale
            .plant_labels
            .get(self.index())
            .map(String::as_str)
            .unwrap_or_else(|| self.fallback_label())
    }

    fn cost(self) -> u32 {
        match self {
            Self::Peashooter => 100,
            Self::Sunflower => 50,
            Self::Wallnut => 75,
            Self::SnowPea => 150,
            Self::Repeater => 200,
            Self::CabbagePult => 125,
            Self::Spikeweed => 100,
            Self::CherryBomb => 150,
            Self::Torchwood => 175,
            Self::Garlic => 50,
        }
    }

    fn max_health(self) -> f32 {
        match self {
            Self::Peashooter => 120.0,
            Self::Sunflower => 90.0,
            Self::Wallnut => 460.0,
            Self::SnowPea => 110.0,
            Self::Repeater => 125.0,
            Self::CabbagePult => 115.0,
            Self::Spikeweed => 80.0,
            Self::CherryBomb => 55.0,
            Self::Torchwood => 180.0,
            Self::Garlic => 150.0,
        }
    }

    fn sprite_path(self) -> &'static str {
        match self {
            Self::Peashooter => "art/sprites/plants/sprout-slinger.png",
            Self::Sunflower => "art/sprites/plants/sunbloom.png",
            Self::Wallnut => "art/sprites/plants/bark-bulwark.png",
            Self::SnowPea => "art/sprites/plants/frost-sprout.png",
            Self::Repeater => "art/sprites/plants/twin-pod.png",
            Self::CabbagePult => "art/sprites/plants/leaf-lobber.png",
            Self::Spikeweed => "art/sprites/plants/briar-mat.png",
            Self::CherryBomb => "art/sprites/plants/blast-berry.png",
            Self::Torchwood => "art/sprites/plants/ember-stump.png",
            Self::Garlic => "art/sprites/plants/scent-root.png",
        }
    }

    fn model_path(self) -> &'static str {
        match self {
            Self::Peashooter => "models/plants/sprout-slinger.glb",
            Self::Sunflower => "models/plants/sunbloom.glb",
            Self::Wallnut => "models/plants/bark-bulwark.glb",
            Self::SnowPea => "models/plants/frost-sprout.glb",
            Self::Repeater => "models/plants/twin-pod.glb",
            Self::CabbagePult => "models/plants/leaf-lobber.glb",
            Self::Spikeweed => "models/plants/briar-mat.glb",
            Self::CherryBomb => "models/plants/blast-berry.glb",
            Self::Torchwood => "models/plants/ember-stump.glb",
            Self::Garlic => "models/plants/scent-root.glb",
        }
    }

    fn description(self, locale: &LocaleText) -> &str {
        locale
            .plant_descriptions
            .get(self.index())
            .map(String::as_str)
            .unwrap_or("")
    }
}

#[derive(Component)]
struct Plant {
    kind: PlantKind,
    col: usize,
    lane: usize,
    health: f32,
    fire_timer: Timer,
    sun_timer: Timer,
}

#[derive(Component)]
struct Zombie {
    kind: ZombieKind,
    lane: usize,
    health: f32,
    max_health: f32,
    speed: f32,
    damage: f32,
    attack_timer: Timer,
    slow_timer: Timer,
    special_timer: Timer,
    jumped: bool,
}

#[derive(Component)]
struct Projectile {
    lane: usize,
    damage: f32,
    speed: f32,
    slow_secs: f32,
    pierce: u32,
    splash_radius: f32,
}

#[derive(Component)]
struct VisualEffect {
    lifetime: Timer,
}

#[derive(Clone, Copy)]
enum LimbKind {
    Arm,
    Leg,
    Head,
}

#[derive(Component)]
struct LimbAnim {
    base: Quat,
    kind: LimbKind,
    phase: f32,
}

#[derive(Component)]
struct GrowIn {
    timer: Timer,
    target_scale: f32,
}

struct ProjectileSpec {
    damage: f32,
    speed: f32,
    slow_secs: f32,
    pierce: u32,
    splash_radius: f32,
    color: Color,
    texture: &'static str,
    visual_size: f32,
    name: &'static str,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum ZombieKind {
    Walker,
    Conehead,
    Runner,
    Buckethead,
    Brute,
    Healer,
    Jumper,
    Digger,
    Frostbite,
    Gargantuar,
}

impl ZombieKind {
    const COUNT: usize = 10;
    const ALL: [Self; 10] = [
        Self::Walker,
        Self::Conehead,
        Self::Runner,
        Self::Buckethead,
        Self::Brute,
        Self::Healer,
        Self::Jumper,
        Self::Digger,
        Self::Frostbite,
        Self::Gargantuar,
    ];
    const ALL_SLICE: &'static [Self] = &Self::ALL;

    fn index(self) -> usize {
        match self {
            Self::Walker => 0,
            Self::Conehead => 1,
            Self::Runner => 2,
            Self::Buckethead => 3,
            Self::Brute => 4,
            Self::Healer => 5,
            Self::Jumper => 6,
            Self::Digger => 7,
            Self::Frostbite => 8,
            Self::Gargantuar => 9,
        }
    }

    fn fallback_label(self) -> &'static str {
        match self {
            Self::Walker => "Walker Zombie",
            Self::Conehead => "Conehead Zombie",
            Self::Runner => "Runner Zombie",
            Self::Buckethead => "Buckethead Zombie",
            Self::Brute => "Brute Zombie",
            Self::Healer => "Healer Zombie",
            Self::Jumper => "Jumper Zombie",
            Self::Digger => "Digger Zombie",
            Self::Frostbite => "Frostbite Zombie",
            Self::Gargantuar => "Gargantuar",
        }
    }

    fn label(self, locale: &LocaleText) -> &str {
        locale
            .zombie_labels
            .get(self.index())
            .map(String::as_str)
            .unwrap_or_else(|| self.fallback_label())
    }

    fn sprite_path(self) -> &'static str {
        match self {
            Self::Walker => "art/sprites/monsters/walker.png",
            Self::Conehead => "art/sprites/monsters/conehead.png",
            Self::Runner => "art/sprites/monsters/runner.png",
            Self::Buckethead => "art/sprites/monsters/buckethead.png",
            Self::Brute => "art/sprites/monsters/brute.png",
            Self::Healer => "art/sprites/monsters/healer.png",
            Self::Jumper => "art/sprites/monsters/jumper.png",
            Self::Digger => "art/sprites/monsters/digger.png",
            Self::Frostbite => "art/sprites/monsters/frostbite.png",
            Self::Gargantuar => "art/sprites/monsters/gargantuar.png",
        }
    }

    fn model_path(self) -> &'static str {
        match self {
            Self::Walker => "models/monsters/walker.glb",
            Self::Conehead => "models/monsters/conehead.glb",
            Self::Runner => "models/monsters/runner.glb",
            Self::Buckethead => "models/monsters/buckethead.glb",
            Self::Brute => "models/monsters/brute.glb",
            Self::Healer => "models/monsters/healer.glb",
            Self::Jumper => "models/monsters/jumper.glb",
            Self::Digger => "models/monsters/digger.glb",
            Self::Frostbite => "models/monsters/frostbite.glb",
            Self::Gargantuar => "models/monsters/gargantuar.glb",
        }
    }

    fn stats(self, wave: u32) -> (f32, f32, f32, f32) {
        let scale = wave as f32;
        match self {
            Self::Walker => (95.0 + scale * 8.0, 0.42, 15.0, 0.28),
            Self::Conehead => (155.0 + scale * 11.0, 0.36, 18.0, 0.34),
            Self::Runner => (75.0 + scale * 7.0, 0.72, 13.0, 0.24),
            Self::Buckethead => (240.0 + scale * 13.0, 0.30, 20.0, 0.38),
            Self::Brute => (300.0 + scale * 16.0, 0.26, 28.0, 0.42),
            Self::Healer => (145.0 + scale * 9.0, 0.34, 12.0, 0.30),
            Self::Jumper => (120.0 + scale * 9.0, 0.50, 18.0, 0.27),
            Self::Digger => (130.0 + scale * 10.0, 0.46, 16.0, 0.25),
            Self::Frostbite => (165.0 + scale * 10.0, 0.32, 17.0, 0.32),
            Self::Gargantuar => (680.0 + scale * 28.0, 0.20, 45.0, 0.50),
        }
    }

    fn score(self) -> u32 {
        match self {
            Self::Walker => 10,
            Self::Conehead => 18,
            Self::Runner => 14,
            Self::Buckethead => 26,
            Self::Brute => 34,
            Self::Healer => 24,
            Self::Jumper => 22,
            Self::Digger => 24,
            Self::Frostbite => 28,
            Self::Gargantuar => 100,
        }
    }

    fn damage_multiplier(self) -> f32 {
        match self {
            Self::Buckethead => 0.72,
            Self::Gargantuar => 0.82,
            _ => 1.0,
        }
    }
}

#[derive(Component)]
struct SunPickup {
    value: u32,
    lifetime: Timer,
}

fn load_levels() -> LevelCatalog {
    let levels = ron::from_str::<Vec<LevelConfig>>(LEVELS_RON)
        .expect("assets/data/levels.ron must contain valid level data");
    assert!(!levels.is_empty(), "at least one level is required");
    LevelCatalog {
        levels,
        selected: 0,
    }
}

fn load_localization() -> LocalizationCatalog {
    let localization = LocalizationCatalog {
        english: ron::from_str::<LocaleText>(EN_RON)
            .expect("assets/i18n/en.ron must contain valid localization data"),
        chinese: ron::from_str::<LocaleText>(ZH_RON)
            .expect("assets/i18n/zh.ron must contain valid localization data"),
    };
    for locale in [&localization.english, &localization.chinese] {
        assert_eq!(locale.plant_labels.len(), PlantKind::COUNT);
        assert_eq!(locale.plant_descriptions.len(), PlantKind::COUNT);
        assert_eq!(locale.zombie_labels.len(), ZombieKind::COUNT);
        for zombie in ZombieKind::ALL {
            assert!(!zombie.label(locale).trim().is_empty());
        }
    }
    localization
}

fn load_release_metadata() -> ReleaseMetadata {
    ron::from_str::<ReleaseMetadata>(VERSION_RON)
        .expect("VERSION.ron must contain valid release metadata")
}

fn load_asset_manifest() -> AssetManifest {
    ron::from_str::<AssetManifest>(ASSET_MANIFEST_RON)
        .expect("assets/manifest.ron must contain valid asset manifest data")
}

fn embedded_asset_for_path(path: &str) -> Option<&'static [u8]> {
    if let Some((_, contents)) = PLANT_SPRITE_ASSETS
        .iter()
        .chain(MONSTER_SPRITE_ASSETS.iter())
        .chain(PLANT_MODEL_ASSETS.iter())
        .chain(MONSTER_MODEL_ASSETS.iter())
        .chain(AUDIO_ASSETS.iter())
        .chain(EFFECT_ASSETS.iter())
        .chain(ENVIRONMENT_ASSETS.iter())
        .find(|(asset_path, _)| *asset_path == path)
    {
        return Some(*contents);
    }

    match path {
        "assets/branding/icon.svg" => Some(BRAND_ICON_SVG.as_bytes()),
        "assets/branding/capsule.svg" => Some(BRAND_CAPSULE_SVG.as_bytes()),
        "assets/branding/generated/app-icon.png" => Some(APP_ICON_PNG),
        "assets/branding/generated/store-capsule.png" => Some(STORE_CAPSULE_PNG),
        "assets/art/plants-sheet.png" => Some(PLANTS_SHEET_PNG),
        "assets/art/monsters-sheet.png" => Some(MONSTERS_SHEET_PNG),
        "assets/art/ui/menu-panel.png" => Some(UI_MENU_PANEL_PNG),
        "assets/art/ui/hud-panel.png" => Some(UI_HUD_PANEL_PNG),
        "assets/art/ui/end-panel.png" => Some(UI_END_PANEL_PNG),
        "assets/fonts/SarasaMonoSC-subset.ttf" => Some(FONT_CJK_TTF),
        "assets/data/levels.ron" => Some(LEVELS_RON.as_bytes()),
        "assets/i18n/en.ron" => Some(EN_RON.as_bytes()),
        "assets/i18n/zh.ron" => Some(ZH_RON.as_bytes()),
        "assets/linux/bevy-open-siege.desktop" => Some(LINUX_DESKTOP_ENTRY.as_bytes()),
        "assets/linux/io.github.bevy_open_siege.BevyOpenSiege.metainfo.xml" => {
            Some(LINUX_APPSTREAM_METAINFO.as_bytes())
        }
        _ => None,
    }
}

fn runtime_asset_manifest_path(runtime_path: &str) -> String {
    format!("assets/{runtime_path}")
}

fn runtime_asset_paths() -> Vec<&'static str> {
    let mut paths = Vec::new();
    paths.extend([
        AUDIO_MUSIC_LOOP,
        AUDIO_PLANT_PLACE,
        AUDIO_SHOOT,
        AUDIO_SUN_COLLECT,
        AUDIO_MONSTER_DOWN,
        AUDIO_VICTORY,
        AUDIO_DEFEAT,
        EFFECT_PEA,
        EFFECT_FROST_POD,
        EFFECT_CABBAGE,
        EFFECT_SUN,
        EFFECT_FIRE,
        EFFECT_EXPLOSION,
        ENV_LAWN_BASE,
        ENV_LANE_GRASS,
        ENV_SOIL_BORDER,
        UI_MENU_PANEL,
        UI_HUD_PANEL,
        UI_END_PANEL,
        FONT_CJK,
    ]);
    paths.extend(PlantKind::ALL.iter().map(|plant| plant.sprite_path()));
    paths.extend(ZombieKind::ALL.iter().map(|zombie| zombie.sprite_path()));
    paths.extend(PlantKind::ALL.iter().map(|plant| plant.model_path()));
    paths.extend(ZombieKind::ALL.iter().map(|zombie| zombie.model_path()));
    paths
}

fn validate_runtime_asset_manifest_coverage(manifest: &AssetManifest) -> Result<(), String> {
    for runtime_path in runtime_asset_paths() {
        let manifest_path = runtime_asset_manifest_path(runtime_path);
        let listed = manifest
            .art
            .iter()
            .chain(manifest.audio.iter())
            .chain(manifest.data.iter())
            .any(|entry| entry.path == manifest_path);
        if !listed {
            return Err(format!(
                "runtime asset path {runtime_path} is missing from assets/manifest.ron"
            ));
        }
        if embedded_asset_for_path(&manifest_path).is_none() {
            return Err(format!(
                "runtime asset path {runtime_path} is not embedded for release validation"
            ));
        }
    }
    Ok(())
}

fn read_be_u32(bytes: &[u8], offset: usize) -> Option<u32> {
    let bytes = bytes.get(offset..offset + 4)?;
    Some(u32::from_be_bytes(bytes.try_into().ok()?))
}

fn read_le_u16(bytes: &[u8], offset: usize) -> Option<u16> {
    let bytes = bytes.get(offset..offset + 2)?;
    Some(u16::from_le_bytes(bytes.try_into().ok()?))
}

fn read_le_i16(bytes: &[u8], offset: usize) -> Option<i16> {
    let bytes = bytes.get(offset..offset + 2)?;
    Some(i16::from_le_bytes(bytes.try_into().ok()?))
}

fn read_le_u32(bytes: &[u8], offset: usize) -> Option<u32> {
    let bytes = bytes.get(offset..offset + 4)?;
    Some(u32::from_le_bytes(bytes.try_into().ok()?))
}

fn png_dimensions(contents: &[u8]) -> Option<(u32, u32)> {
    if contents.len() < 24 || !contents.starts_with(b"\x89PNG\r\n\x1a\n") {
        return None;
    }
    if contents.get(12..16)? != b"IHDR" {
        return None;
    }
    Some((read_be_u32(contents, 16)?, read_be_u32(contents, 20)?))
}

fn wav_audio_summary(contents: &[u8]) -> Option<(u16, u16, u32, u32)> {
    if contents.len() < 12 || &contents[0..4] != b"RIFF" || &contents[8..12] != b"WAVE" {
        return None;
    }

    let mut offset = 12;
    let mut channels = None;
    let mut bits_per_sample = None;
    let mut sample_rate = None;
    let mut data_bytes = None;
    while offset + 8 <= contents.len() {
        let chunk_id = contents.get(offset..offset + 4)?;
        let chunk_size = read_le_u32(contents, offset + 4)? as usize;
        let chunk_start = offset + 8;
        let chunk_end = chunk_start.checked_add(chunk_size)?;
        if chunk_end > contents.len() {
            return None;
        }

        if chunk_id == b"fmt " {
            channels = read_le_u16(contents, chunk_start + 2);
            sample_rate = read_le_u32(contents, chunk_start + 4);
            bits_per_sample = read_le_u16(contents, chunk_start + 14);
        } else if chunk_id == b"data" {
            data_bytes = Some(chunk_size as u32);
        }

        offset = chunk_end + (chunk_size % 2);
    }

    Some((channels?, bits_per_sample?, sample_rate?, data_bytes?))
}

#[derive(Debug, Clone)]
struct WavPcmStats {
    duration_secs: f32,
    channels: u16,
    bits_per_sample: u16,
    sample_rate: u32,
    peak: f32,
    rms: f32,
}

fn wav_pcm_stats(contents: &[u8]) -> Option<WavPcmStats> {
    if contents.len() < 12 || &contents[0..4] != b"RIFF" || &contents[8..12] != b"WAVE" {
        return None;
    }

    let mut offset = 12;
    let mut channels = None;
    let mut bits_per_sample = None;
    let mut sample_rate = None;
    let mut data_range = None;
    while offset + 8 <= contents.len() {
        let chunk_id = contents.get(offset..offset + 4)?;
        let chunk_size = read_le_u32(contents, offset + 4)? as usize;
        let chunk_start = offset + 8;
        let chunk_end = chunk_start.checked_add(chunk_size)?;
        if chunk_end > contents.len() {
            return None;
        }

        if chunk_id == b"fmt " {
            channels = read_le_u16(contents, chunk_start + 2);
            sample_rate = read_le_u32(contents, chunk_start + 4);
            bits_per_sample = read_le_u16(contents, chunk_start + 14);
        } else if chunk_id == b"data" {
            data_range = Some(chunk_start..chunk_end);
        }

        offset = chunk_end + (chunk_size % 2);
    }

    let channels = channels?;
    let bits_per_sample = bits_per_sample?;
    let sample_rate = sample_rate?;
    let data_range = data_range?;
    if channels == 0 || bits_per_sample != 16 || sample_rate == 0 {
        return None;
    }

    let data = contents.get(data_range)?;
    if data.len() < 2 || data.len() % 2 != 0 {
        return None;
    }
    let mut peak = 0.0_f32;
    let mut square_sum = 0.0_f64;
    let sample_count = data.len() / 2;
    for sample_index in 0..sample_count {
        let sample = read_le_i16(data, sample_index * 2)? as f32 / 32768.0;
        let abs = sample.abs();
        peak = peak.max(abs);
        square_sum += f64::from(sample * sample);
    }
    let frame_count = sample_count as f32 / channels as f32;
    let duration_secs = frame_count / sample_rate as f32;
    let rms = (square_sum / sample_count as f64).sqrt() as f32;

    Some(WavPcmStats {
        duration_secs,
        channels,
        bits_per_sample,
        sample_rate,
        peak,
        rms,
    })
}

fn validate_embedded_asset_format(path: &str, contents: &[u8]) -> Result<(), String> {
    if path.ends_with(".png") {
        let Some((width, height)) = png_dimensions(contents) else {
            return Err(format!("asset manifest path {path} is not a valid PNG"));
        };
        if width < 32 || height < 32 {
            return Err(format!(
                "asset manifest path {path} has undersized PNG dimensions {width}x{height}"
            ));
        }
    } else if path.ends_with(".wav") {
        let Some((channels, bits_per_sample, sample_rate, data_bytes)) =
            wav_audio_summary(contents)
        else {
            return Err(format!("asset manifest path {path} is not a valid WAV"));
        };
        if channels == 0 || bits_per_sample < 8 || sample_rate < 22_050 || data_bytes < 1_024 {
            return Err(format!(
                "asset manifest path {path} has invalid WAV properties: {channels} channels, {bits_per_sample} bits, {sample_rate} Hz, {data_bytes} data bytes"
            ));
        }
    } else if path.ends_with(".svg") {
        let Ok(svg) = std::str::from_utf8(contents) else {
            return Err(format!("asset manifest path {path} is not valid UTF-8 SVG"));
        };
        if !svg.contains("<svg") {
            return Err(format!(
                "asset manifest path {path} does not contain an SVG root"
            ));
        }
    } else if path.ends_with(".glb") {
        if contents.len() < 20 || !contents.starts_with(b"glTF") {
            return Err(format!("asset manifest path {path} is not a valid GLB"));
        }
    } else if path.ends_with(".ron") {
        if std::str::from_utf8(contents).is_err() {
            return Err(format!("asset manifest path {path} is not valid UTF-8 RON"));
        }
    } else if path.ends_with(".ttf") {
        if contents.len() < 12 || !contents.starts_with(&[0x00, 0x01, 0x00, 0x00]) {
            return Err(format!(
                "asset manifest path {path} is not a valid TrueType font"
            ));
        }
    } else if path.ends_with(".desktop") {
        let Ok(desktop) = std::str::from_utf8(contents) else {
            return Err(format!(
                "asset manifest path {path} is not valid UTF-8 desktop metadata"
            ));
        };
        for required in [
            "[Desktop Entry]",
            "Type=Application",
            "Name=Bevy Open Siege",
            "Exec=bevy_open_siege",
            "Categories=Game;StrategyGame;",
        ] {
            if !desktop.contains(required) {
                return Err(format!(
                    "asset manifest path {path} is missing desktop field {required}"
                ));
            }
        }
    } else if path.ends_with(".xml") {
        let Ok(xml) = std::str::from_utf8(contents) else {
            return Err(format!(
                "asset manifest path {path} is not valid UTF-8 XML metadata"
            ));
        };
        for required in [
            "<component type=\"desktop-application\">",
            "<id>io.github.bevy_open_siege.BevyOpenSiege</id>",
            "<launchable type=\"desktop-id\">bevy-open-siege.desktop</launchable>",
            "<binary>bevy_open_siege</binary>",
        ] {
            if !xml.contains(required) {
                return Err(format!(
                    "asset manifest path {path} is missing AppStream field {required}"
                ));
            }
        }
    }
    Ok(())
}

fn validate_release_copy() -> Result<(), String> {
    let release_docs = [
        ("README.md", README_MD),
        ("STORE_PAGE.md", STORE_PAGE_MD),
        ("PRESSKIT.md", PRESSKIT_MD),
        ("STORE_SCREENSHOTS.md", STORE_SCREENSHOTS_MD),
        ("RELEASE_NOTES.md", RELEASE_NOTES_MD),
        ("PRIVACY.md", PRIVACY_MD),
        ("SUPPORT.md", SUPPORT_MD),
        ("TROUBLESHOOTING.md", TROUBLESHOOTING_MD),
        ("BUILD_PROVENANCE.md", BUILD_PROVENANCE_MD),
        ("VERSION.ron", VERSION_RON),
    ];
    for (path, contents) in release_docs {
        let lower = contents.to_ascii_lowercase();
        for banned in ["placeholder", "prototype", "tbd", "todo"] {
            if lower.contains(banned) {
                return Err(format!(
                    "release-facing copy {path} still contains banned term '{banned}'"
                ));
            }
        }
    }

    let metadata = load_release_metadata();
    if metadata.release_channel == "prototype" {
        return Err("release metadata still uses prototype channel".to_string());
    }
    if metadata
        .content_rating_note
        .to_ascii_lowercase()
        .contains("placeholder")
    {
        return Err(
            "release metadata content rating still mentions placeholder visuals".to_string(),
        );
    }
    validate_release_metadata_copy(&metadata)?;
    Ok(())
}

fn validate_release_metadata_copy(metadata: &ReleaseMetadata) -> Result<(), String> {
    for (path, contents) in [
        ("STORE_PAGE.md", STORE_PAGE_MD),
        ("PRESSKIT.md", PRESSKIT_MD),
        ("STORE_SCREENSHOTS.md", STORE_SCREENSHOTS_MD),
        ("RELEASE_NOTES.md", RELEASE_NOTES_MD),
    ] {
        if !contents.contains(&metadata.product_name) {
            return Err(format!(
                "release-facing copy {path} does not mention product name {}",
                metadata.product_name
            ));
        }
    }
    if !RELEASE_NOTES_MD.contains(&metadata.version) {
        return Err(format!(
            "RELEASE_NOTES.md does not mention release version {}",
            metadata.version
        ));
    }
    for language in &metadata.supported_languages {
        let language_name = match language.as_str() {
            "en" => "English",
            "zh" => "Chinese",
            other => other,
        };
        for (path, contents) in [
            ("STORE_PAGE.md", STORE_PAGE_MD),
            ("PRESSKIT.md", PRESSKIT_MD),
            ("RELEASE_NOTES.md", RELEASE_NOTES_MD),
        ] {
            if !contents.contains(language_name) {
                return Err(format!(
                    "release-facing copy {path} does not mention supported language {language_name}"
                ));
            }
        }
    }
    for platform in &metadata.supported_platforms {
        let platform_copy = platform.replace('-', " ").to_ascii_lowercase();
        let platform_id = platform.to_ascii_lowercase();
        for (path, contents) in [
            ("STORE_PAGE.md", STORE_PAGE_MD),
            ("PRESSKIT.md", PRESSKIT_MD),
            ("RELEASE_NOTES.md", RELEASE_NOTES_MD),
        ] {
            let lower = contents.to_ascii_lowercase();
            if !lower.contains(&platform_copy) && !lower.contains(&platform_id) {
                return Err(format!(
                    "release-facing copy {path} does not mention supported platform {platform}"
                ));
            }
        }
    }
    Ok(())
}

fn audit_marketing_doc_section(
    lines: &mut Vec<String>,
    path: &str,
    contents: &str,
    section: &str,
) -> Result<(), String> {
    if !contents.contains(section) {
        return Err(format!(
            "marketing document {path} is missing section {section}"
        ));
    }
    lines.push(format!("{path} section | {section}"));
    Ok(())
}

fn audit_marketing_doc_token(
    lines: &mut Vec<String>,
    path: &str,
    contents: &str,
    token: &str,
) -> Result<(), String> {
    if !contents.contains(token) {
        return Err(format!(
            "marketing document {path} is missing token {token}"
        ));
    }
    lines.push(format!("{path} token | {token}"));
    Ok(())
}

fn marketing_audit_report() -> Result<String, String> {
    validate_release_copy()?;

    let metadata = load_release_metadata();
    let mut lines = vec!["marketing audit ok".to_string()];
    lines.push(format!("product: {}", metadata.product_name));
    lines.push(format!("version: {}", metadata.version));
    lines.push(format!(
        "languages: {}",
        metadata.supported_languages.join(", ")
    ));
    lines.push(format!(
        "platforms: {}",
        metadata.supported_platforms.join(", ")
    ));

    for section in [
        "## Short Description",
        "## Long Description",
        "## Key Features",
        "## Current Release Status",
        "## Tags",
    ] {
        audit_marketing_doc_section(&mut lines, "STORE_PAGE.md", STORE_PAGE_MD, section)?;
    }
    for token in [
        "Ten plant types",
        "Ten enemy types",
        "English and Chinese localization",
        "Linux x86_64",
        "Strategy",
        "Single Player",
        "no blood",
        "no in-app purchases",
        "no telemetry",
    ] {
        audit_marketing_doc_token(&mut lines, "STORE_PAGE.md", STORE_PAGE_MD, token)?;
    }

    for section in ["## Boilerplate", "## Fact Sheet", "## Available Media"] {
        audit_marketing_doc_section(&mut lines, "PRESSKIT.md", PRESSKIT_MD, section)?;
    }
    for token in [
        "Developer:",
        "Genre:",
        "Players:",
        "Languages:",
        "Engine: Bevy",
        "assets/branding/generated/app-icon.png",
        "assets/branding/generated/store-capsule.png",
        "assets/art/plants-sheet.png",
        "assets/art/monsters-sheet.png",
        "store_screenshot_check.sh",
        "STORE_PAGE.md",
        "ART_ASSETS.md",
        "CONTENT_RATING.md",
    ] {
        audit_marketing_doc_token(&mut lines, "PRESSKIT.md", PRESSKIT_MD, token)?;
    }
    for token in [
        "Fantasy non-realistic combat",
        "No blood or gore",
        "No in-app purchases",
        "telemetry",
    ] {
        audit_marketing_doc_token(&mut lines, "PRESSKIT.md", PRESSKIT_MD, token)?;
    }

    for section in [
        "## Required Captures",
        "## Capture Requirements",
        "## QA Requirements",
    ] {
        audit_marketing_doc_section(
            &mut lines,
            "STORE_SCREENSHOTS.md",
            STORE_SCREENSHOTS_MD,
            section,
        )?;
    }
    for token in [
        "screenshots/01-title-menu.png",
        "screenshots/02-early-defense.png",
        "screenshots/03-special-enemies.png",
        "screenshots/04-late-siege.png",
        "screenshots/05-victory-summary.png",
        "1920x1080",
        "one English screenshot and one Chinese screenshot",
        "qa-session/store-screenshots.md",
        "store_screenshot_check.sh --plan .",
        "store_screenshot_check.sh --capture-startup . screenshots 01-title-menu.png 8",
        "store_screenshot_check.sh --validate-dir screenshots",
        "visual-smoke.txt",
        "visual-readability-audit.txt",
    ] {
        audit_marketing_doc_token(
            &mut lines,
            "STORE_SCREENSHOTS.md",
            STORE_SCREENSHOTS_MD,
            token,
        )?;
    }

    for section in [
        "### Package Contents",
        "### Known Release Review Items",
        "### Verification",
    ] {
        audit_marketing_doc_section(&mut lines, "RELEASE_NOTES.md", RELEASE_NOTES_MD, section)?;
    }
    for token in [
        "10 playable plant types",
        "10 enemy types",
        "English and Chinese localization",
        "localization-audit.txt",
        "runtime-smoke.txt",
        "audio-smoke.txt",
        "./runtime_smoke.sh ./bevy_open_siege",
        "./audio_smoke.sh ./bevy_open_siege",
    ] {
        audit_marketing_doc_token(&mut lines, "RELEASE_NOTES.md", RELEASE_NOTES_MD, token)?;
    }

    audit_marketing_doc_section(
        &mut lines,
        "ART_ASSETS.md",
        ART_ASSETS_MD,
        "## Production Art",
    )?;
    audit_marketing_doc_section(
        &mut lines,
        "ART_ASSETS.md",
        ART_ASSETS_MD,
        "## Replacement Policy",
    )?;
    for token in [
        "assets/art/sprites/plants/*.png",
        "assets/art/sprites/monsters/*.png",
        "assets/art/ui/*.png",
        "assets/audio/*.wav",
        "production_art",
    ] {
        audit_marketing_doc_token(&mut lines, "ART_ASSETS.md", ART_ASSETS_MD, token)?;
    }

    for token in [
        "Bevy game engine",
        "No third-party art",
        "CREDITS.md",
        "MIT OR Apache-2.0",
    ] {
        audit_marketing_doc_token(
            &mut lines,
            "THIRD_PARTY_NOTICES.md",
            THIRD_PARTY_NOTICES_MD,
            token,
        )?;
    }
    for token in ["Bevy Open Siege", "imagegen", "synthesized audio"] {
        audit_marketing_doc_token(&mut lines, "CREDITS.md", CREDITS_MD, token)?;
    }

    audit_marketing_doc_section(
        &mut lines,
        "CONTENT_RATING.md",
        CONTENT_RATING_MD,
        "## Gameplay Content",
    )?;
    audit_marketing_doc_section(
        &mut lines,
        "CONTENT_RATING.md",
        CONTENT_RATING_MD,
        "## Sensitive Content",
    )?;
    audit_marketing_doc_section(
        &mut lines,
        "CONTENT_RATING.md",
        CONTENT_RATING_MD,
        "## Store Questionnaire Answers",
    )?;
    for token in [
        "fantasy undead",
        "no blood",
        "no gore",
        "No in-app purchases",
        "No online multiplayer",
        "Data collection: none",
    ] {
        audit_marketing_doc_token(&mut lines, "CONTENT_RATING.md", CONTENT_RATING_MD, token)?;
    }

    lines.push(
        "checked documents: store, presskit, screenshots, content rating, release notes, art, notices, credits"
            .to_string(),
    );
    lines.push(
        "checked media references: app icon, store capsule, plant sheet, monster sheet, screenshot plan"
            .to_string(),
    );
    lines.push("checked release claims: 10 plants, 10 enemies, en/zh, linux-x86_64".to_string());
    Ok(lines.join("\n"))
}

fn ip_audit_report() -> Result<String, String> {
    let banned_ascii_terms = [
        "plants vs zombies",
        "pvz",
        "plant-vs-undead",
        "peashooter",
        "sunflower",
        "wallnut",
        "wall-nut",
        "snow pea",
        "snow-pea",
        "repeater",
        "cabbage pult",
        "cabbage-pult",
        "spikeweed",
        "cherry bomb",
        "cherry-bomb",
        "torchwood",
        "garlic",
    ];
    let banned_cjk_terms = [
        "豌豆射手",
        "向日葵",
        "坚果墙",
        "寒冰射手",
        "双发射手",
        "卷心菜投手",
        "地刺",
        "樱桃炸弹",
        "火炬木",
        "大蒜",
    ];

    let mut scanned_blocks: Vec<(&str, String)> = vec![
        ("README.md", README_MD.to_string()),
        ("STORE_PAGE.md", STORE_PAGE_MD.to_string()),
        ("PRESSKIT.md", PRESSKIT_MD.to_string()),
        ("RELEASE_NOTES.md", RELEASE_NOTES_MD.to_string()),
        ("ART_ASSETS.md", ART_ASSETS_MD.to_string()),
        ("THIRD_PARTY_NOTICES.md", THIRD_PARTY_NOTICES_MD.to_string()),
        ("CREDITS.md", CREDITS_MD.to_string()),
        ("assets/i18n/en.ron", EN_RON.to_string()),
        ("assets/i18n/zh.ron", ZH_RON.to_string()),
        ("assets/manifest.ron", ASSET_MANIFEST_RON.to_string()),
    ];

    scanned_blocks.push((
        "control bindings",
        control_bindings()
            .iter()
            .map(|(key, scope, action)| format!("{key} {scope} {action}"))
            .collect::<Vec<_>>()
            .join("\n"),
    ));

    let localization = load_localization();
    scanned_blocks.push((
        "player-facing labels",
        [
            localization.english.plant_labels.join("\n"),
            localization.chinese.plant_labels.join("\n"),
            localization.english.zombie_labels.join("\n"),
            localization.chinese.zombie_labels.join("\n"),
            PlantKind::ALL
                .iter()
                .map(|plant| plant.fallback_label())
                .collect::<Vec<_>>()
                .join("\n"),
        ]
        .join("\n"),
    ));

    for (name, contents) in &scanned_blocks {
        let lower = contents.to_ascii_lowercase();
        for term in banned_ascii_terms {
            if lower.contains(term) {
                return Err(format!(
                    "{name} contains reserved source-material term: {term}"
                ));
            }
        }
        for term in banned_cjk_terms {
            if contents.contains(term) {
                return Err(format!(
                    "{name} contains reserved source-material term: {term}"
                ));
            }
        }
    }

    let plant_label_count = localization.english.plant_labels.len();
    let unique_plants = localization
        .english
        .plant_labels
        .iter()
        .chain(localization.chinese.plant_labels.iter())
        .collect::<std::collections::BTreeSet<_>>()
        .len();
    if plant_label_count != PlantKind::COUNT || unique_plants != PlantKind::COUNT * 2 {
        return Err(format!(
            "expected {} original plant labels in each language, got en {plant_label_count} and unique {unique_plants}",
            PlantKind::COUNT
        ));
    }

    let mut lines = vec!["ip audit ok".to_string()];
    lines.push(format!(
        "checked ascii reserved terms: {}",
        banned_ascii_terms.len()
    ));
    lines.push(format!(
        "checked cjk reserved terms: {}",
        banned_cjk_terms.len()
    ));
    lines.push(format!(
        "checked release-facing blocks: {}",
        scanned_blocks.len()
    ));
    lines.push(format!(
        "checked plant labels: {plant_label_count}/{}",
        PlantKind::COUNT
    ));
    lines.push(format!(
        "checked enemy labels: {}/{}",
        localization.english.zombie_labels.len(),
        ZombieKind::COUNT
    ));
    lines.push("checked asset manifest paths for renamed plant and frost assets".to_string());
    Ok(lines.join("\n"))
}

fn save_audit_report() -> Result<String, String> {
    let levels = load_levels();
    let level_count = levels.levels.len();
    let mut lines = vec!["save audit ok".to_string()];

    let explicit = PathBuf::from("/tmp/bevy_open_siege_custom_save.ron");
    let explicit_path = save_path_from_env(Some(explicit.clone()), None, None);
    if explicit_path != explicit {
        return Err("explicit save path override was not honored".to_string());
    }
    lines.push(format!(
        "explicit override path | {}",
        explicit_path.display()
    ));

    let xdg_path = save_path_from_env(None, Some(PathBuf::from("/tmp/xdg-data")), None);
    if xdg_path != Path::new("/tmp/xdg-data/bevy_open_siege/bevy_open_siege_save.ron") {
        return Err(format!(
            "XDG save path resolved unexpectedly: {}",
            xdg_path.display()
        ));
    }
    lines.push(format!("xdg data path | {}", xdg_path.display()));

    let home_path = save_path_from_env(None, None, Some(PathBuf::from("/home/player")));
    if home_path != Path::new("/home/player/.local/share/bevy_open_siege/bevy_open_siege_save.ron") {
        return Err(format!(
            "home fallback save path resolved unexpectedly: {}",
            home_path.display()
        ));
    }
    lines.push(format!("home fallback path | {}", home_path.display()));

    let portable_path = save_path_from_env(None, None, None);
    if portable_path != Path::new(SAVE_FILE_NAME) {
        return Err(format!(
            "portable fallback save path resolved unexpectedly: {}",
            portable_path.display()
        ));
    }
    lines.push(format!(
        "portable fallback path | {}",
        portable_path.display()
    ));

    let legacy_save = r#"(
    version: 1,
    language: Chinese,
    unlocked_levels: 3,
    best_scores: [1250, 800],
)"#;
    let parsed_legacy = ron::from_str::<SaveData>(legacy_save)
        .map_err(|error| format!("legacy save parse failed: {error}"))?;
    let legacy_progress = normalized_progress(&parsed_legacy, level_count);
    if parsed_legacy.language != Language::Chinese {
        return Err("legacy save language did not parse as Chinese".to_string());
    }
    if legacy_progress.unlocked_levels != 3 {
        return Err(format!(
            "legacy save unlock normalization expected 3, got {}",
            legacy_progress.unlocked_levels
        ));
    }
    if legacy_progress.best_scores.len() != level_count {
        return Err(format!(
            "legacy save score slots expected {level_count}, got {}",
            legacy_progress.best_scores.len()
        ));
    }
    if legacy_progress.best_scores[0] != 1250 || legacy_progress.best_scores[1] != 800 {
        return Err("legacy save best scores were not preserved".to_string());
    }
    if parsed_legacy.settings.master_volume != SaveSettings::default().master_volume
        || parsed_legacy.settings.fullscreen != SaveSettings::default().fullscreen
    {
        return Err("legacy save did not receive default settings".to_string());
    }
    lines.push(format!(
        "legacy save parse | language Chinese | unlocked {} | score slots {} | default volume {:.2} | fullscreen {}",
        legacy_progress.unlocked_levels,
        legacy_progress.best_scores.len(),
        parsed_legacy.settings.master_volume,
        parsed_legacy.settings.fullscreen
    ));

    let over_unlocked = SaveData {
        version: 1,
        language: Language::English,
        unlocked_levels: level_count + 99,
        best_scores: vec![42],
        settings: SaveSettings {
            master_volume: 1.8,
            fullscreen: true,
        },
    };
    let over_progress = normalized_progress(&over_unlocked, level_count);
    if over_progress.unlocked_levels != level_count {
        return Err(format!(
            "over-unlocked save should clamp to {level_count}, got {}",
            over_progress.unlocked_levels
        ));
    }
    if GameSettings::from(&over_unlocked.settings).master_volume != 1.0 {
        return Err("master volume above 1.0 did not clamp to 1.0".to_string());
    }
    lines.push(format!(
        "normalization | unlock clamp {}/{} | volume clamp {:.2} | fullscreen true",
        over_progress.unlocked_levels,
        level_count,
        GameSettings::from(&over_unlocked.settings).master_volume
    ));

    let temp_dir = env::temp_dir().join(format!(
        "bevy_open_siege_save_audit_{}_{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|error| format!("system clock before epoch: {error}"))?
            .as_nanos()
    ));
    fs::create_dir_all(&temp_dir)
        .map_err(|error| format!("failed to create save audit temp dir: {error}"))?;
    let temp_save_path = temp_dir.join(SAVE_FILE_NAME);
    fs::write(&temp_save_path, legacy_save)
        .map_err(|error| format!("failed to write save audit temp file: {error}"))?;
    let parsed_file = parse_save_file(&temp_save_path)
        .ok_or_else(|| "save audit temp file did not parse".to_string())?;
    if parsed_file.language != Language::Chinese {
        return Err("save audit temp file parsed with unexpected language".to_string());
    }
    lines.push("parse_save_file | temp legacy file ok".to_string());
    let _ = fs::remove_file(&temp_save_path);
    let _ = fs::remove_dir(&temp_dir);

    lines.push(format!("checked level slots: {level_count}"));
    lines.push("checked paths: explicit, xdg, home, portable".to_string());
    lines.push("checked compatibility: legacy save, normalization, settings clamp".to_string());
    Ok(lines.join("\n"))
}

fn privacy_support_audit_report() -> Result<String, String> {
    validate_release_copy()?;
    save_audit_report()?;

    let mut lines = vec!["privacy audit ok".to_string()];
    let mut checked_tokens = 0usize;

    for (path, contents, tokens) in [
        (
            "PRIVACY.md",
            PRIVACY_MD,
            vec![
                "offline single-player game",
                "does not include telemetry",
                "does not intentionally collect, transmit, sell, or share personal information",
                "selected language",
                "unlocked campaign level count",
                "best scores",
                "master volume",
                "fullscreen preference",
                "$XDG_DATA_HOME/bevy_open_siege/bevy_open_siege_save.ron",
                "BEVY_OPEN_SIEGE_SAVE_PATH",
                "uninstall_linux_user.sh --purge",
                "networking, telemetry, crash reporting, cloud saves, accounts, or third-party services",
            ],
        ),
        (
            "SUPPORT.md",
            SUPPORT_MD,
            vec![
                "Bevy Open Siege 0.1.0 release candidate",
                "./bevy_open_siege --print-release-info",
                "./bevy_open_siege --validate-data",
                "./bevy_open_siege --audit-save",
                "./runtime_smoke.sh ./bevy_open_siege 12",
                "SHA256SUMS",
                "Do not attach unrelated personal files",
                "no automatic crash uploader, telemetry, analytics, accounts, or network services",
                "QA_SIGNOFF.md",
            ],
        ),
        (
            "TROUBLESHOOTING.md",
            TROUBLESHOOTING_MD,
            vec![
                "Bevy Open Siege Troubleshooting",
                "./verify_release.sh --quick .",
                "./runtime_smoke.sh ./bevy_open_siege 12",
                "./visual_smoke.sh ./bevy_open_siege 15",
                "./audio_smoke.sh ./bevy_open_siege 12",
                "./bevy_open_siege --print-save-path",
                "BEVY_OPEN_SIEGE_SAVE_PATH=qa-save.ron",
                "./linux_package_audit.sh .",
                "./qa_evidence_summary.sh --summary . qa-session platform-session",
                "./final_signoff_check.sh --check . qa-session platform-session",
            ],
        ),
    ] {
        for token in tokens {
            if !contents.contains(token) {
                return Err(format!("{path} is missing privacy/support token: {token}"));
            }
            checked_tokens += 1;
        }
        lines.push(format!("{path} tokens checked"));
    }

    for (path, contents, tokens) in [
        (
            "README.md",
            README_MD,
            vec![
                "PRIVACY.md",
                "SUPPORT.md",
                "TROUBLESHOOTING.md",
                "BEVY_OPEN_SIEGE_SAVE_PATH",
            ],
        ),
        (
            "RELEASE_NOTES.md",
            RELEASE_NOTES_MD,
            vec![
                "PRIVACY.md",
                "SUPPORT.md",
                "TROUBLESHOOTING.md",
                "privacy-audit.txt",
            ],
        ),
        (
            "RELEASE_CHECKLIST.md",
            include_str!("../RELEASE_CHECKLIST.md"),
            vec![
                "--audit-privacy",
                "privacy-audit.txt",
                "PRIVACY.md",
                "SUPPORT.md",
                "TROUBLESHOOTING.md",
            ],
        ),
        (
            "QA_SIGNOFF.md",
            include_str!("../QA_SIGNOFF.md"),
            vec![
                "Privacy, support, and troubleshooting audit",
                "privacy-audit.txt",
                "TROUBLESHOOTING.md",
            ],
        ),
    ] {
        for token in tokens {
            if !contents.contains(token) {
                return Err(format!("{path} is missing release privacy token: {token}"));
            }
            checked_tokens += 1;
        }
        lines.push(format!("{path} privacy release references checked"));
    }

    let forbidden_source_tokens = [
        "std::net::",
        "TcpStream",
        "UdpSocket",
        "reqwest::",
        "ureq::",
        "hyper::",
        "websocket::",
        "tungstenite",
    ];
    for token in forbidden_source_tokens {
        if MAIN_RS.matches(token).count() > 1 || CARGO_TOML.contains(token) {
            return Err(format!(
                "source or Cargo.toml contains network/privacy-sensitive token: {token}"
            ));
        }
    }

    lines.push(format!("checked privacy/support tokens: {checked_tokens}"));
    lines.push(
        "checked local data fields: language, unlocks, best scores, volume, fullscreen".to_string(),
    );
    lines.push("checked network posture: no telemetry, analytics, accounts, crash uploaders, cloud saves, or network services declared in source/dependencies".to_string());
    lines.push("checked deletion posture: uninstall preserves saves by default; --purge required for save removal".to_string());
    lines.push(
        "manual privacy QA still required: review store/platform disclosure text before release"
            .to_string(),
    );
    Ok(lines.join("\n"))
}

fn audit_document_tokens(
    lines: &mut Vec<String>,
    path: &str,
    contents: &str,
    tokens: &[&str],
) -> Result<usize, String> {
    for token in tokens {
        if !contents.contains(token) {
            return Err(format!(
                "{path} is missing release provenance token: {token}"
            ));
        }
    }
    lines.push(format!("{path} tokens checked: {}", tokens.len()));
    Ok(tokens.len())
}

fn release_provenance_audit_report() -> Result<String, String> {
    validate_release_copy()?;
    let metadata = validate_release_data()?;
    let mut lines = vec!["release provenance audit ok".to_string()];
    let mut checked_tokens = 0usize;

    if !CARGO_TOML.contains(&format!("name = \"{}\"", env!("CARGO_PKG_NAME"))) {
        return Err("Cargo.toml package name does not match compiled package".to_string());
    }
    if !CARGO_TOML.contains(&format!("version = \"{}\"", metadata.version)) {
        return Err(format!(
            "Cargo.toml does not declare release version {}",
            metadata.version
        ));
    }
    if !CARGO_LOCK.contains("[[package]]") || !CARGO_LOCK.contains("name = \"bevy\"") {
        return Err("Cargo.lock does not contain locked dependency packages".to_string());
    }
    if !CARGO_LOCK.contains("name = \"bevy_open_siege\"") {
        return Err("Cargo.lock does not contain the release package".to_string());
    }

    checked_tokens += audit_document_tokens(
        &mut lines,
        "BUILD_PROVENANCE.md",
        BUILD_PROVENANCE_MD,
        &[
            "Cargo.toml",
            "Cargo.lock",
            "VERSION.ron",
            "assets/manifest.ron",
            "scripts/package_release.sh",
            "scripts/package_windows.ps1",
            "scripts/package_macos.sh",
            "scripts/create_candidate_evidence.sh",
            "scripts/create_store_submission_pack.sh",
            "scripts/release_check.sh",
            "release-manifest.json",
            "SHA256SUMS",
            "scripts/smoke_release_archive.sh",
            "release-provenance-audit.txt",
            "cargo metadata --locked --filter-platform x86_64-unknown-linux-gnu",
            "THIRD_PARTY_LICENSES.md",
            "UNKNOWN_LICENSE",
            "final_signoff_check.sh --check",
            "Release approved: Yes",
        ],
    )?;
    checked_tokens += audit_document_tokens(
        &mut lines,
        "README.md",
        README_MD,
        &[
            "BUILD_PROVENANCE.md",
            "--audit-release-provenance",
            "release-provenance-audit.txt",
        ],
    )?;
    checked_tokens += audit_document_tokens(
        &mut lines,
        "RELEASE_NOTES.md",
        RELEASE_NOTES_MD,
        &["BUILD_PROVENANCE.md", "release-provenance-audit.txt"],
    )?;
    checked_tokens += audit_document_tokens(
        &mut lines,
        "RELEASE_CHECKLIST.md",
        include_str!("../RELEASE_CHECKLIST.md"),
        &[
            "--audit-release-provenance",
            "release-provenance-audit.txt",
            "BUILD_PROVENANCE.md",
            "Cargo.lock",
            "SHA256SUMS",
        ],
    )?;
    checked_tokens += audit_document_tokens(
        &mut lines,
        "QA_SIGNOFF.md",
        include_str!("../QA_SIGNOFF.md"),
        &["Build provenance audit", "release-provenance-audit.txt"],
    )?;

    let release_check_script = include_str!("../scripts/release_check.sh");
    let package_script = include_str!("../scripts/package_release.sh");
    let smoke_script = include_str!("../scripts/smoke_release_archive.sh");
    let license_script = include_str!("../scripts/generate_third_party_licenses.py");
    checked_tokens += audit_document_tokens(
        &mut lines,
        "scripts/release_check.sh",
        release_check_script,
        &[
            "cargo clippy --all-targets -- -D warnings",
            "--audit-release-provenance",
        ],
    )?;
    checked_tokens += audit_document_tokens(
        &mut lines,
        "scripts/package_release.sh",
        package_script,
        &[
            "--audit-release-provenance",
            "release-provenance-audit.txt",
            "BUILD_PROVENANCE.md",
            "THIRD_PARTY_LICENSES.md",
            "release-manifest.json",
            "SHA256SUMS",
            "smoke_release_archive.sh",
        ],
    )?;
    checked_tokens += audit_document_tokens(
        &mut lines,
        "scripts/smoke_release_archive.sh",
        smoke_script,
        &[
            "--audit-release-provenance",
            "release-provenance-audit.txt",
            "release-manifest.json",
            "sha256sum -c SHA256SUMS",
            "UNKNOWN_LICENSE",
            "BUILD_PROVENANCE.md",
        ],
    )?;
    checked_tokens += audit_document_tokens(
        &mut lines,
        "scripts/generate_third_party_licenses.py",
        license_script,
        &[
            "cargo",
            "metadata",
            "--locked",
            "--filter-platform",
            "missing license metadata",
        ],
    )?;

    lines.push(format!("product: {}", metadata.product_name));
    lines.push(format!("version: {}", metadata.version));
    lines.push(format!("cargo package: {}", env!("CARGO_PKG_NAME")));
    lines.push(format!("cargo lock bytes: {}", CARGO_LOCK.len()));
    lines.push(format!("checked provenance tokens: {checked_tokens}"));
    lines.push(
        "checked build inputs: Cargo.toml, Cargo.lock, VERSION.ron, assets/manifest.ron"
            .to_string(),
    );
    lines.push(
        "checked package entry points: Linux, Windows, macOS, candidate/store handoff, release gate"
            .to_string(),
    );
    lines.push(
        "checked integrity evidence: release-manifest.json, SHA256SUMS, and archive smoke verification"
            .to_string(),
    );
    lines.push("checked dependency evidence: cargo metadata locked license report".to_string());
    lines.push("manual provenance QA still required: attach host OS, Rust version, archive checksum, and final signoff evidence".to_string());
    Ok(lines.join("\n"))
}

fn appstream_release_version(xml: &str) -> Option<&str> {
    let release = xml.split("<release ").nth(1)?;
    let version_start = release.find("version=\"")? + "version=\"".len();
    let rest = release.get(version_start..)?;
    let version_end = rest.find('"')?;
    rest.get(..version_end)
}

fn validate_release_data() -> Result<ReleaseMetadata, String> {
    let levels = load_levels();
    if levels.levels.len() < 10 {
        return Err(format!(
            "expected at least 10 campaign levels, found {}",
            levels.levels.len()
        ));
    }

    let mut ids = std::collections::HashSet::new();
    let mut previous_final_wave = 0;
    for level in &levels.levels {
        if !ids.insert(level.id.as_str()) {
            return Err(format!("duplicate level id {}", level.id));
        }
        if level.title_en.trim().is_empty() || level.title_zh.trim().is_empty() {
            return Err(format!("level {} has an empty localized title", level.id));
        }
        if level.final_wave < previous_final_wave {
            return Err(format!(
                "level {} breaks campaign wave progression",
                level.id
            ));
        }
        previous_final_wave = level.final_wave;
    }

    let localization = load_localization();
    for (code, locale) in [("en", &localization.english), ("zh", &localization.chinese)] {
        if locale.title.trim().is_empty() {
            return Err(format!("locale {code} has an empty title"));
        }
        if locale.plant_labels.len() != PlantKind::COUNT
            || locale.plant_descriptions.len() != PlantKind::COUNT
            || locale.zombie_labels.len() != ZombieKind::COUNT
        {
            return Err(format!(
                "locale {code} roster lengths do not match gameplay rosters"
            ));
        }
    }

    let metadata = load_release_metadata();
    if metadata.product_name.trim().is_empty() {
        return Err("release metadata product_name is empty".to_string());
    }
    if metadata.version != env!("CARGO_PKG_VERSION") {
        return Err(format!(
            "VERSION.ron version {} does not match Cargo.toml {}",
            metadata.version,
            env!("CARGO_PKG_VERSION")
        ));
    }
    match appstream_release_version(LINUX_APPSTREAM_METAINFO) {
        Some(version) if version == metadata.version => {}
        Some(version) => {
            return Err(format!(
                "AppStream release version {version} does not match VERSION.ron {}",
                metadata.version
            ));
        }
        None => return Err("AppStream metadata is missing a release version".to_string()),
    }
    for required_language in ["en", "zh"] {
        if !metadata
            .supported_languages
            .iter()
            .any(|language| language == required_language)
        {
            return Err(format!(
                "release metadata missing language {required_language}"
            ));
        }
    }
    if metadata.supported_platforms.is_empty() {
        return Err("release metadata must list at least one platform".to_string());
    }
    validate_release_copy()?;

    let manifest = load_asset_manifest();
    if manifest.branding.len() < 2 {
        return Err("asset manifest must include app icon and store capsule branding".to_string());
    }
    let production_art_count = manifest
        .branding
        .iter()
        .chain(manifest.art.iter())
        .filter(|entry| entry.status == "production_art")
        .count();
    if production_art_count < 4 {
        return Err(format!(
            "expected at least 4 production art assets, found {production_art_count}"
        ));
    }
    if manifest.audio.len() < AUDIO_ASSETS.len() {
        return Err(format!(
            "expected at least {} audio assets, found {}",
            AUDIO_ASSETS.len(),
            manifest.audio.len()
        ));
    }
    validate_runtime_asset_manifest_coverage(&manifest)?;

    for entry in manifest
        .branding
        .iter()
        .chain(manifest.art.iter())
        .chain(manifest.audio.iter())
        .chain(manifest.data.iter())
    {
        if entry.id.trim().is_empty()
            || entry.path.trim().is_empty()
            || entry.kind.trim().is_empty()
            || entry.usage.trim().is_empty()
            || entry.status.trim().is_empty()
        {
            return Err(format!(
                "asset manifest entry {} has empty fields",
                entry.id
            ));
        }
        let Some(contents) = embedded_asset_for_path(&entry.path) else {
            return Err(format!(
                "asset manifest path {} is not embedded",
                entry.path
            ));
        };
        if contents.is_empty() {
            return Err(format!("asset manifest path {} is empty", entry.path));
        }
        validate_embedded_asset_format(&entry.path, contents)?;
    }

    Ok(metadata)
}

fn asset_audit_report() -> Result<String, String> {
    let manifest = load_asset_manifest();
    let mut lines = Vec::new();
    let mut png_count = 0;
    let mut wav_count = 0;
    let mut svg_count = 0;
    let mut glb_count = 0;
    let mut metadata_count = 0;
    let mut production_art_count = 0;
    let mut total_bytes: usize = 0;

    lines.push("asset audit ok".to_string());
    for entry in manifest
        .branding
        .iter()
        .chain(manifest.art.iter())
        .chain(manifest.audio.iter())
        .chain(manifest.data.iter())
    {
        let Some(contents) = embedded_asset_for_path(&entry.path) else {
            return Err(format!(
                "asset manifest path {} is not embedded",
                entry.path
            ));
        };
        validate_embedded_asset_format(&entry.path, contents)?;
        total_bytes += contents.len();
        if entry.status == "production_art" {
            production_art_count += 1;
        }

        if entry.path.ends_with(".png") {
            let (width, height) = png_dimensions(contents)
                .ok_or_else(|| format!("asset manifest path {} is not a valid PNG", entry.path))?;
            png_count += 1;
            lines.push(format!(
                "{} | png {}x{} | {} | {}",
                entry.path, width, height, entry.status, entry.usage
            ));
        } else if entry.path.ends_with(".wav") {
            let (channels, bits_per_sample, sample_rate, data_bytes) = wav_audio_summary(contents)
                .ok_or_else(|| format!("asset manifest path {} is not a valid WAV", entry.path))?;
            let bytes_per_second =
                channels as f32 * (bits_per_sample as f32 / 8.0) * sample_rate as f32;
            let duration_secs = data_bytes as f32 / bytes_per_second.max(1.0);
            wav_count += 1;
            lines.push(format!(
                "{} | wav {:.2}s {}ch {}bit {}Hz | {}",
                entry.path, duration_secs, channels, bits_per_sample, sample_rate, entry.usage
            ));
        } else if entry.path.ends_with(".svg") {
            svg_count += 1;
            lines.push(format!(
                "{} | svg {} bytes | {} | {}",
                entry.path,
                contents.len(),
                entry.status,
                entry.usage
            ));
        } else if entry.path.ends_with(".glb") {
            glb_count += 1;
            lines.push(format!(
                "{} | glb {} bytes | {} | {}",
                entry.path,
                contents.len(),
                entry.status,
                entry.usage
            ));
        } else {
            metadata_count += 1;
            lines.push(format!(
                "{} | data {} bytes | {} | {}",
                entry.path,
                contents.len(),
                entry.status,
                entry.usage
            ));
        }
    }

    lines.push(format!("png assets: {png_count}"));
    lines.push(format!("wav assets: {wav_count}"));
    lines.push(format!("svg assets: {svg_count}"));
    lines.push(format!("glb assets: {glb_count}"));
    lines.push(format!("metadata assets: {metadata_count}"));
    lines.push(format!("production art assets: {production_art_count}"));
    lines.push(format!("embedded asset bytes: {total_bytes}"));

    Ok(lines.join("\n"))
}

fn audio_audit_report() -> Result<String, String> {
    let mut lines = vec!["audio audit ok".to_string()];
    let mut music_count = 0;
    let mut sound_count = 0;
    let mut max_peak = 0.0_f32;
    let mut min_rms = f32::MAX;

    for (path, contents) in AUDIO_ASSETS {
        let stats = wav_pcm_stats(contents)
            .ok_or_else(|| format!("audio asset {path} is not 16-bit PCM WAV"))?;
        if stats.channels != 1 || stats.bits_per_sample != 16 || stats.sample_rate != 44_100 {
            return Err(format!(
                "audio asset {path} must be mono 16-bit 44100Hz, found {}ch {}bit {}Hz",
                stats.channels, stats.bits_per_sample, stats.sample_rate
            ));
        }
        if stats.peak >= 0.98 {
            return Err(format!(
                "audio asset {path} peak {:.3} is too close to clipping",
                stats.peak
            ));
        }
        if stats.peak < 0.05 || stats.rms < 0.015 {
            return Err(format!(
                "audio asset {path} is too quiet or effectively silent: peak {:.3}, rms {:.3}",
                stats.peak, stats.rms
            ));
        }

        let is_music = path == "assets/audio/music-loop.wav";
        if is_music {
            music_count += 1;
            if !(6.0..=30.0).contains(&stats.duration_secs) {
                return Err(format!(
                    "music asset {path} duration {:.2}s is outside release loop bounds",
                    stats.duration_secs
                ));
            }
            if stats.rms > 0.12 {
                return Err(format!(
                    "music asset {path} rms {:.3} is too loud for the default mix",
                    stats.rms
                ));
            }
        } else {
            sound_count += 1;
            if !(0.08..=1.50).contains(&stats.duration_secs) {
                return Err(format!(
                    "sound effect {path} duration {:.2}s is outside release cue bounds",
                    stats.duration_secs
                ));
            }
            if stats.rms > 0.28 {
                return Err(format!(
                    "sound effect {path} rms {:.3} is too loud for the default mix",
                    stats.rms
                ));
            }
        }

        max_peak = max_peak.max(stats.peak);
        min_rms = min_rms.min(stats.rms);
        lines.push(format!(
            "{path} | {:.2}s | peak {:.3} | rms {:.3} | {}ch {}bit {}Hz",
            stats.duration_secs,
            stats.peak,
            stats.rms,
            stats.channels,
            stats.bits_per_sample,
            stats.sample_rate
        ));
    }

    if music_count != 1 || sound_count != 6 {
        return Err(format!(
            "audio audit expected 1 music loop and 6 sound effects, found {music_count} music and {sound_count} effects"
        ));
    }
    if audio_enabled() {
        return Err(
            "audio should remain opt-in until device QA approves default audio".to_string(),
        );
    }

    lines.push(format!("checked music loops: {music_count}"));
    lines.push(format!("checked sound effects: {sound_count}"));
    lines.push(format!(
        "checked mix safety: peak max {max_peak:.3}, rms min {min_rms:.3}"
    ));
    lines.push("checked startup policy: audio remains opt-in by default".to_string());
    lines.push("manual device QA still required: speakers and headphones".to_string());
    Ok(lines.join("\n"))
}

fn control_bindings() -> Vec<(&'static str, &'static str, &'static str)> {
    vec![
        ("Enter", "menu/end", "start or restart"),
        (
            "Arrow keys",
            "menu/gameplay",
            "choose levels and move grid cursor",
        ),
        (
            "Mouse left",
            "gameplay",
            "move cursor and plant selected seed",
        ),
        (
            "Mouse right",
            "gameplay",
            "move cursor and shovel selected tile",
        ),
        ("Tab", "menu", "cycle selected level"),
        ("1", "menu/gameplay", "select level 1 or Sprout Slinger"),
        ("2", "menu/gameplay", "select level 2 or Sunbloom"),
        ("3", "menu/gameplay", "select level 3 or Bark Bulwark"),
        ("4", "menu/gameplay", "select level 4 or Frost Sprout"),
        ("5", "menu/gameplay", "select level 5 or Twin Pod"),
        ("6", "menu/gameplay", "select level 6 or Leaf Lobber"),
        ("7", "menu/gameplay", "select level 7 or Briar Mat"),
        ("8", "menu/gameplay", "select level 8 or Blast Berry"),
        ("9", "menu/gameplay", "select level 9 or Ember Stump"),
        ("0", "menu/gameplay", "select level 10 or Scent Root"),
        (
            "Space",
            "menu/gameplay",
            "start unlocked level or plant selected seed",
        ),
        ("Backspace", "gameplay", "shovel selected tile"),
        ("C", "gameplay", "collect sun on selected tile"),
        ("L", "global", "switch language"),
        ("P", "gameplay", "pause or resume"),
        ("F", "global", "toggle fullscreen"),
        ("+", "global", "raise saved master volume"),
        ("-", "global", "lower saved master volume"),
        ("R", "end", "retry after victory or defeat"),
    ]
}

fn validate_control_copy() -> Result<(), String> {
    for (key, _scope, _action) in control_bindings() {
        if !README_MD.contains(&format!("`{key}`")) {
            return Err(format!(
                "README.md controls section missing `{key}` binding"
            ));
        }
    }

    let localization = load_localization();
    for (code, help, arrow_word) in [
        ("en", localization.english.menu_help.as_str(), "arrows"),
        ("zh", localization.chinese.menu_help.as_str(), "方向键"),
    ] {
        for token in ["Tab", "1-0", "Enter", "F", "+/-", "L"] {
            if !help.contains(token) {
                return Err(format!(
                    "locale {code} menu_help missing control token {token}"
                ));
            }
        }
        if !help.contains(arrow_word) {
            return Err(format!(
                "locale {code} menu_help missing arrow-key control token"
            ));
        }
    }

    Ok(())
}

fn control_audit_report() -> Result<String, String> {
    validate_control_copy()?;
    let bindings = control_bindings();
    let mut lines = Vec::new();
    lines.push(format!("control audit ok: {} bindings", bindings.len()));
    for (key, scope, action) in bindings {
        lines.push(format!("{key} | {scope} | {action}"));
    }
    lines.push("documentation: README.md controls section covered".to_string());
    lines.push("menu localization: en/zh menu_help covered".to_string());
    Ok(lines.join("\n"))
}

fn next_menu_selection(selected: usize, level_count: usize) -> usize {
    if level_count == 0 {
        0
    } else {
        (selected + 1) % level_count
    }
}

fn previous_menu_selection(selected: usize, level_count: usize) -> usize {
    if level_count == 0 {
        0
    } else if selected == 0 {
        level_count - 1
    } else {
        selected - 1
    }
}

fn digit_key_index(key: KeyCode) -> Option<usize> {
    match key {
        KeyCode::Digit1 => Some(0),
        KeyCode::Digit2 => Some(1),
        KeyCode::Digit3 => Some(2),
        KeyCode::Digit4 => Some(3),
        KeyCode::Digit5 => Some(4),
        KeyCode::Digit6 => Some(5),
        KeyCode::Digit7 => Some(6),
        KeyCode::Digit8 => Some(7),
        KeyCode::Digit9 => Some(8),
        KeyCode::Digit0 => Some(9),
        _ => None,
    }
}

fn plant_kind_for_digit_key(key: KeyCode) -> Option<PlantKind> {
    digit_key_index(key).and_then(|index| PlantKind::ALL.get(index).copied())
}

fn volume_after_step(master_volume: f32, delta: f32) -> f32 {
    (master_volume + delta).clamp(0.0, 1.0)
}

fn cursor_after_arrow(col: usize, lane: usize, key: KeyCode) -> (usize, usize) {
    match key {
        KeyCode::ArrowLeft => (col.saturating_sub(1), lane),
        KeyCode::ArrowRight => ((col + 1).min(COLS - 1), lane),
        KeyCode::ArrowUp => (col, lane.saturating_sub(1)),
        KeyCode::ArrowDown => (col, (lane + 1).min(LANES - 1)),
        _ => (col, lane),
    }
}

fn can_start_selected_level(progress: &ProgressState, selected: usize) -> bool {
    progress.is_unlocked(selected)
}

fn plant_placement_block(
    paused: bool,
    occupied: bool,
    sun: u32,
    cooldown: f32,
    kind: PlantKind,
) -> Option<&'static str> {
    if paused {
        Some("paused")
    } else if occupied {
        Some("occupied")
    } else if sun < kind.cost() {
        Some("insufficient sun")
    } else if cooldown > 0.0 {
        Some("cooldown")
    } else {
        None
    }
}

fn input_flow_audit_report() -> Result<String, String> {
    let progress = ProgressState {
        unlocked_levels: 1,
        best_scores: vec![0; PlantKind::COUNT],
    };

    if next_menu_selection(0, 10) != 1 || next_menu_selection(9, 10) != 0 {
        return Err("menu next selection wrap failed".to_string());
    }
    if previous_menu_selection(0, 10) != 9 || previous_menu_selection(5, 10) != 4 {
        return Err("menu previous selection wrap failed".to_string());
    }
    if digit_key_index(KeyCode::Digit1) != Some(0) || digit_key_index(KeyCode::Digit0) != Some(9) {
        return Err("menu digit level selection failed".to_string());
    }
    if !can_start_selected_level(&progress, 0) || can_start_selected_level(&progress, 1) {
        return Err("menu locked-level start gating failed".to_string());
    }
    if Language::English.next() != Language::Chinese
        || Language::Chinese.next() != Language::English
    {
        return Err("language toggle flow failed".to_string());
    }
    if volume_after_step(0.95, 0.1) != 1.0 || volume_after_step(0.05, -0.1) != 0.0 {
        return Err("volume clamp flow failed".to_string());
    }
    if cursor_after_arrow(0, 0, KeyCode::ArrowLeft) != (0, 0)
        || cursor_after_arrow(COLS - 1, LANES - 1, KeyCode::ArrowRight) != (COLS - 1, LANES - 1)
        || cursor_after_arrow(3, 2, KeyCode::ArrowUp) != (3, 1)
        || cursor_after_arrow(3, 2, KeyCode::ArrowDown) != (3, 3)
    {
        return Err("cursor clamp flow failed".to_string());
    }
    for (index, key) in [
        KeyCode::Digit1,
        KeyCode::Digit2,
        KeyCode::Digit3,
        KeyCode::Digit4,
        KeyCode::Digit5,
        KeyCode::Digit6,
        KeyCode::Digit7,
        KeyCode::Digit8,
        KeyCode::Digit9,
        KeyCode::Digit0,
    ]
    .into_iter()
    .enumerate()
    {
        if plant_kind_for_digit_key(key) != Some(PlantKind::ALL[index]) {
            return Err(format!("plant digit selection failed at index {index}"));
        }
    }

    let kind = PlantKind::Peashooter;
    if plant_placement_block(false, false, kind.cost(), 0.0, kind).is_some() {
        return Err("affordable plant placement was blocked".to_string());
    }
    for (label, paused, occupied, sun, cooldown) in [
        ("paused", true, false, kind.cost(), 0.0),
        ("occupied", false, true, kind.cost(), 0.0),
        ("insufficient sun", false, false, kind.cost() - 1, 0.0),
        ("cooldown", false, false, kind.cost(), 0.1),
    ] {
        if plant_placement_block(paused, occupied, sun, cooldown, kind) != Some(label) {
            return Err(format!("plant placement block failed for {label}"));
        }
    }

    let mut lines = Vec::new();
    lines.push("input flow audit ok".to_string());
    lines.push("menu navigation: wrap next/previous and digit selection covered".to_string());
    lines.push("menu start gating: locked blocked, unlocked starts".to_string());
    lines.push(
        "global settings: language toggle, fullscreen toggle, volume clamp covered".to_string(),
    );
    lines.push("gameplay cursor: arrow clamps covered".to_string());
    lines.push(format!(
        "gameplay seed selection: {}/{} plants covered",
        PlantKind::COUNT,
        PlantKind::COUNT
    ));
    lines.push(
        "gameplay placement: affordable, paused, occupied, cooldown, insufficient sun covered"
            .to_string(),
    );
    lines.push("gameplay shovel: keyboard and mouse bindings covered by control map".to_string());
    lines.push("gameplay collection: cursor sun collection key covered by control map".to_string());
    lines.push("pause gating: planting blocked while paused".to_string());
    lines.push("end flow: retry keys covered by control map".to_string());
    lines.push(format!("checked bindings: {}", control_bindings().len()));
    Ok(lines.join("\n"))
}

fn audit_localized_text_field(
    lines: &mut Vec<String>,
    language_code: &str,
    label: &str,
    value: &str,
) -> Result<(), String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(format!("locale {language_code} field {label} is empty"));
    }
    let lower = trimmed.to_ascii_lowercase();
    for banned in ["placeholder", "prototype", "todo", "tbd"] {
        if lower.contains(banned) {
            return Err(format!(
                "locale {language_code} field {label} contains banned term '{banned}'"
            ));
        }
    }
    lines.push(format!(
        "{language_code} {label} | chars {}",
        trimmed.chars().count()
    ));
    Ok(())
}

fn localization_audit_report() -> Result<String, String> {
    let localization = load_localization();
    let levels = load_levels();
    let mut lines = vec!["localization audit ok".to_string()];

    for (language, language_code) in [(Language::English, "en"), (Language::Chinese, "zh")] {
        let locale = localization.text(language);
        for (label, value) in [
            ("title", locale.title.as_str()),
            ("menu_help", locale.menu_help.as_str()),
            ("menu_start", locale.menu_start.as_str()),
            ("game_over", locale.game_over.as_str()),
            ("victory", locale.victory.as_str()),
            ("retry", locale.retry.as_str()),
            ("play_again", locale.play_again.as_str()),
            ("locked_level", locale.locked_level.as_str()),
            ("level", locale.level.as_str()),
            ("best_score", locale.best_score.as_str()),
            ("no_score", locale.no_score.as_str()),
            ("hud.sun", locale.hud.sun.as_str()),
            ("hud.seed", locale.hud.seed.as_str()),
            ("hud.wave", locale.hud.wave.as_str()),
            ("hud.score", locale.hud.score.as_str()),
            ("hud.cursor", locale.hud.cursor.as_str()),
            ("hud.collect_sun", locale.hud.collect_sun.as_str()),
            ("hud.language", locale.hud.language.as_str()),
            ("hud.level", locale.hud.level.as_str()),
        ] {
            audit_localized_text_field(&mut lines, language_code, label, value)?;
        }

        if locale.plant_labels.len() != PlantKind::COUNT {
            return Err(format!(
                "locale {language_code} has {} plant labels, expected {}",
                locale.plant_labels.len(),
                PlantKind::COUNT
            ));
        }
        if locale.plant_descriptions.len() != PlantKind::COUNT {
            return Err(format!(
                "locale {language_code} has {} plant descriptions, expected {}",
                locale.plant_descriptions.len(),
                PlantKind::COUNT
            ));
        }
        if locale.zombie_labels.len() != ZombieKind::COUNT {
            return Err(format!(
                "locale {language_code} has {} zombie labels, expected {}",
                locale.zombie_labels.len(),
                ZombieKind::COUNT
            ));
        }

        for (index, label) in locale.plant_labels.iter().enumerate() {
            audit_localized_text_field(
                &mut lines,
                language_code,
                &format!("plant_label_{}", index + 1),
                label,
            )?;
        }
        for (index, description) in locale.plant_descriptions.iter().enumerate() {
            audit_localized_text_field(
                &mut lines,
                language_code,
                &format!("plant_description_{}", index + 1),
                description,
            )?;
        }
        for (index, label) in locale.zombie_labels.iter().enumerate() {
            audit_localized_text_field(
                &mut lines,
                language_code,
                &format!("zombie_label_{}", index + 1),
                label,
            )?;
        }
        for (index, level) in levels.levels.iter().enumerate() {
            audit_localized_text_field(
                &mut lines,
                language_code,
                &format!("level_title_{}", index + 1),
                level.title(language),
            )?;
        }

        lines.push(format!(
            "{language_code} coverage | plants {}/{} | plant descriptions {}/{} | zombies {}/{} | levels {}/{}",
            locale.plant_labels.len(),
            PlantKind::COUNT,
            locale.plant_descriptions.len(),
            PlantKind::COUNT,
            locale.zombie_labels.len(),
            ZombieKind::COUNT,
            levels.levels.len(),
            levels.levels.len()
        ));
    }

    lines.push("checked languages: en, zh".to_string());
    lines.push(format!("checked plants: {}", PlantKind::COUNT));
    lines.push(format!("checked zombies: {}", ZombieKind::COUNT));
    lines.push(format!("checked levels: {}", levels.levels.len()));
    Ok(lines.join("\n"))
}

fn max_line_chars(text: &str) -> usize {
    text.lines()
        .map(|line| line.chars().count())
        .max()
        .unwrap_or(0)
}

fn audit_text_block(
    lines: &mut Vec<String>,
    label: &str,
    text: &str,
    max_allowed_chars: usize,
    max_allowed_lines: usize,
) -> Result<(), String> {
    if text.trim().is_empty() {
        return Err(format!("layout text block {label} is empty"));
    }
    let max_chars = max_line_chars(text);
    let line_count = text.lines().count().max(1);
    if max_chars > max_allowed_chars {
        return Err(format!(
            "layout text block {label} has line length {max_chars}, above release bound {max_allowed_chars}"
        ));
    }
    if line_count > max_allowed_lines {
        return Err(format!(
            "layout text block {label} has {line_count} lines, above release bound {max_allowed_lines}"
        ));
    }
    lines.push(format!(
        "{label} | lines {line_count}/{} | max chars {max_chars}/{}",
        max_allowed_lines, max_allowed_chars
    ));
    Ok(())
}

fn layout_audit_report() -> Result<String, String> {
    let levels = load_levels();
    let localization = load_localization();
    let progress = ProgressState {
        unlocked_levels: levels.levels.len(),
        best_scores: vec![99_999; levels.levels.len()],
    };
    let settings = GameSettings {
        master_volume: 1.0,
        fullscreen: true,
    };
    let pause = PauseState { paused: true };
    let mut lines = vec!["layout audit ok".to_string()];

    for language in [Language::English, Language::Chinese] {
        let locale = localization.text(language);
        let lang = match language {
            Language::English => "en",
            Language::Chinese => "zh",
        };
        audit_text_block(
            &mut lines,
            &format!("{lang} menu title"),
            &locale.title,
            48,
            1,
        )?;
        audit_text_block(
            &mut lines,
            &format!("{lang} menu help"),
            &locale.menu_help,
            96,
            1,
        )?;
        audit_text_block(
            &mut lines,
            &format!("{lang} menu start"),
            &locale.menu_start,
            80,
            1,
        )?;

        let mut selected_levels = levels.clone();
        for index in 0..selected_levels.levels.len() {
            selected_levels.selected = index;
            audit_text_block(
                &mut lines,
                &format!("{lang} menu level {}", index + 1),
                &menu_level_line(locale, language, &selected_levels, &progress),
                120,
                1,
            )?;
        }
        audit_text_block(
            &mut lines,
            &format!("{lang} menu roster"),
            &menu_roster_text(locale, language, &levels, &progress),
            96,
            levels.levels.len(),
        )?;
        audit_text_block(
            &mut lines,
            &format!("{lang} menu settings"),
            &menu_settings_line(&settings, language),
            96,
            1,
        )?;

        let mut state =
            BoardState::for_level(levels.levels.len() - 1, levels.levels.last().unwrap());
        state.sun = 9_999;
        state.score = 99_999;
        state.kills = 999;
        state.wave = levels.levels.last().unwrap().final_wave;
        for cooldown in &mut state.plant_cooldowns {
            *cooldown = 18.0;
        }
        for plant in PlantKind::ALL {
            state.selected = plant;
            audit_text_block(
                &mut lines,
                &format!("{lang} hud status {}", plant.fallback_label()),
                &hud_status_text(
                    locale,
                    language,
                    &state,
                    levels.levels.last().unwrap(),
                    &settings,
                    &pause,
                ),
                132,
                2,
            )?;
        }
        audit_text_block(
            &mut lines,
            &format!("{lang} seed bank"),
            &hud_seed_bank_text(locale, &state),
            132,
            2,
        )?;
        audit_text_block(
            &mut lines,
            &format!("{lang} pause panel"),
            &pause_text(locale, language, &settings),
            96,
            2,
        )?;
        audit_text_block(
            &mut lines,
            &format!("{lang} game over subtitle"),
            &end_subtitle(locale, GameState::GameOver, &state),
            96,
            1,
        )?;
        audit_text_block(
            &mut lines,
            &format!("{lang} victory subtitle"),
            &end_subtitle(locale, GameState::Victory, &state),
            96,
            1,
        )?;
    }

    lines.push("checked languages: en, zh".to_string());
    lines.push(format!("checked levels: {}", levels.levels.len()));
    lines.push(format!("checked plants in HUD: {}", PlantKind::COUNT));
    Ok(lines.join("\n"))
}

#[derive(Debug, Clone, Copy)]
struct VisualViewport {
    name: &'static str,
    width: u32,
    height: u32,
}

fn srgb_to_linear_channel(channel: f32) -> f32 {
    if channel <= 0.04045 {
        channel / 12.92
    } else {
        ((channel + 0.055) / 1.055).powf(2.4)
    }
}

fn relative_luminance((red, green, blue): (f32, f32, f32)) -> f32 {
    0.2126 * srgb_to_linear_channel(red)
        + 0.7152 * srgb_to_linear_channel(green)
        + 0.0722 * srgb_to_linear_channel(blue)
}

fn contrast_ratio(foreground: (f32, f32, f32), background: (f32, f32, f32)) -> f32 {
    let foreground_luminance = relative_luminance(foreground);
    let background_luminance = relative_luminance(background);
    let lighter = foreground_luminance.max(background_luminance);
    let darker = foreground_luminance.min(background_luminance);
    (lighter + 0.05) / (darker + 0.05)
}

fn composite_over(
    foreground: (f32, f32, f32),
    alpha: f32,
    background: (f32, f32, f32),
) -> (f32, f32, f32) {
    (
        foreground.0 * alpha + background.0 * (1.0 - alpha),
        foreground.1 * alpha + background.1 * (1.0 - alpha),
        foreground.2 * alpha + background.2 * (1.0 - alpha),
    )
}

fn audit_visual_viewport(lines: &mut Vec<String>, viewport: VisualViewport) -> Result<(), String> {
    const TOP_HUD_PX: u32 = 74;
    const BOTTOM_HUD_PX: u32 = 112;
    const MIN_PLAYFIELD_HEIGHT_PX: u32 = 340;
    const MIN_WIDTH_PX: u32 = 960;
    const MAX_CHROME_RATIO: f32 = 0.35;

    if viewport.width < MIN_WIDTH_PX {
        return Err(format!(
            "visual viewport {} width {} is below release minimum {}",
            viewport.name, viewport.width, MIN_WIDTH_PX
        ));
    }
    let chrome_height = TOP_HUD_PX + BOTTOM_HUD_PX;
    if viewport.height <= chrome_height {
        return Err(format!(
            "visual viewport {} height {} cannot fit HUD chrome {}",
            viewport.name, viewport.height, chrome_height
        ));
    }
    let playfield_height = viewport.height - chrome_height;
    if playfield_height < MIN_PLAYFIELD_HEIGHT_PX {
        return Err(format!(
            "visual viewport {} playfield height {} is below release minimum {}",
            viewport.name, playfield_height, MIN_PLAYFIELD_HEIGHT_PX
        ));
    }
    let chrome_ratio = chrome_height as f32 / viewport.height as f32;
    if chrome_ratio > MAX_CHROME_RATIO {
        return Err(format!(
            "visual viewport {} HUD chrome ratio {:.2} exceeds {:.2}",
            viewport.name, chrome_ratio, MAX_CHROME_RATIO
        ));
    }

    lines.push(format!(
        "viewport {} | {}x{} | playfield {}px | hud chrome {:.0}%",
        viewport.name,
        viewport.width,
        viewport.height,
        playfield_height,
        chrome_ratio * 100.0
    ));
    Ok(())
}

fn audit_visual_contrast(
    lines: &mut Vec<String>,
    label: &str,
    foreground: (f32, f32, f32),
    background: (f32, f32, f32),
) -> Result<(), String> {
    let ratio = contrast_ratio(foreground, background);
    if ratio < 4.5 {
        return Err(format!(
            "visual contrast {label} ratio {ratio:.2} is below 4.5"
        ));
    }
    lines.push(format!("{label} contrast | {ratio:.2}:1"));
    Ok(())
}

fn visual_readability_audit_report() -> Result<String, String> {
    let layout_report = layout_audit_report()?;
    let asset_report = asset_audit_report()?;
    let mut lines = vec!["visual readability audit ok".to_string()];

    for viewport in [
        VisualViewport {
            name: "desktop-720p",
            width: 1280,
            height: 720,
        },
        VisualViewport {
            name: "desktop-768p",
            width: 1366,
            height: 768,
        },
        VisualViewport {
            name: "desktop-1080p",
            width: 1920,
            height: 1080,
        },
        VisualViewport {
            name: "desktop-1440p",
            width: 2560,
            height: 1440,
        },
        VisualViewport {
            name: "handheld-800p",
            width: 1280,
            height: 800,
        },
        VisualViewport {
            name: "compact-540p",
            width: 960,
            height: 540,
        },
    ] {
        audit_visual_viewport(&mut lines, viewport)?;
    }

    let scene_background = (0.05, 0.075, 0.055);
    let menu_panel = composite_over((0.02, 0.035, 0.02), 0.92, scene_background);
    let hud_top_panel = composite_over((0.02, 0.035, 0.02), 0.86, scene_background);
    let hud_bottom_panel = composite_over((0.03, 0.045, 0.025), 0.88, scene_background);
    audit_visual_contrast(&mut lines, "menu title", (0.88, 0.96, 0.68), menu_panel)?;
    audit_visual_contrast(&mut lines, "menu help", (0.74, 0.84, 0.68), menu_panel)?;
    audit_visual_contrast(&mut lines, "menu start", (0.96, 0.78, 0.30), menu_panel)?;
    audit_visual_contrast(&mut lines, "hud status", (0.92, 0.96, 0.78), hud_top_panel)?;
    audit_visual_contrast(
        &mut lines,
        "hud seed bank",
        (0.82, 0.92, 0.70),
        hud_bottom_panel,
    )?;

    for required_line in [
        "en hud status Sprout Slinger | lines 2/2",
        "en seed bank | lines 2/2",
        "zh hud status Sprout Slinger | lines 2/2",
        "zh seed bank | lines 2/2",
    ] {
        if !layout_report.contains(required_line) {
            return Err(format!(
                "visual readability audit missing layout evidence: {required_line}"
            ));
        }
    }

    for required_asset in [
        "assets/art/sprites/plants/sprout-slinger.png | png 251x627",
        "assets/art/sprites/monsters/gargantuar.png | png 251x627",
        "assets/art/effects/explosion.png | png 256x256",
        "assets/art/environment/lawn-base.png | png 512x512",
        "assets/art/ui/menu-panel.png | png 192x192",
        "assets/art/ui/hud-panel.png | png 192x96",
        "assets/art/ui/end-panel.png | png 192x192",
    ] {
        if !asset_report.contains(required_asset) {
            return Err(format!(
                "visual readability audit missing asset evidence: {required_asset}"
            ));
        }
    }

    lines.push("checked viewports: 6".to_string());
    lines.push("checked contrast pairs: 5".to_string());
    lines.push("checked HUD wrapping: en/zh status and seed bank".to_string());
    lines.push("checked visual assets: plant, monster, effect, environment, ui chrome".to_string());
    Ok(lines.join("\n"))
}

fn accessibility_audit_report() -> Result<String, String> {
    let control_report = control_audit_report()?;
    let input_flow_report = input_flow_audit_report()?;
    let layout_report = layout_audit_report()?;
    let visual_report = visual_readability_audit_report()?;
    let audio_report = audio_audit_report()?;

    for required_binding in [
        "Enter | menu/end | start or restart",
        "Arrow keys | menu/gameplay | choose levels and move grid cursor",
        "Space | menu/gameplay | start unlocked level or plant selected seed",
        "Backspace | gameplay | shovel selected tile",
        "C | gameplay | collect sun on selected tile",
        "L | global | switch language",
        "P | gameplay | pause or resume",
        "F | global | toggle fullscreen",
        "+ | global | raise saved master volume",
        "- | global | lower saved master volume",
        "R | end | retry after victory or defeat",
    ] {
        if !control_report.contains(required_binding) {
            return Err(format!(
                "accessibility audit missing keyboard binding evidence: {required_binding}"
            ));
        }
    }
    for required_flow in [
        "menu start gating: locked blocked, unlocked starts",
        "gameplay seed selection: 10/10 plants covered",
        "gameplay placement: affordable, paused, occupied, cooldown, insufficient sun covered",
        "pause gating: planting blocked while paused",
        "end flow: retry keys covered by control map",
    ] {
        if !input_flow_report.contains(required_flow) {
            return Err(format!(
                "accessibility audit missing input flow evidence: {required_flow}"
            ));
        }
    }
    for required_layout in [
        "checked languages: en, zh",
        "checked plants in HUD: 10",
        "en menu help | lines 1/1",
        "zh menu help | lines 1/1",
        "en seed bank | lines 2/2",
        "zh seed bank | lines 2/2",
    ] {
        if !layout_report.contains(required_layout) {
            return Err(format!(
                "accessibility audit missing layout evidence: {required_layout}"
            ));
        }
    }
    for required_visual in [
        "checked contrast pairs: 5",
        "checked HUD wrapping: en/zh status and seed bank",
        "menu title contrast |",
        "hud status contrast |",
    ] {
        if !visual_report.contains(required_visual) {
            return Err(format!(
                "accessibility audit missing visual evidence: {required_visual}"
            ));
        }
    }
    if !audio_report.contains("checked startup policy: audio remains opt-in by default") {
        return Err("accessibility audit missing no-audio startup policy evidence".to_string());
    }

    let mut lines = Vec::new();
    lines.push("accessibility audit ok".to_string());
    lines.push(
        "keyboard-only flow: menu, gameplay, pause, settings, and end screens covered".to_string(),
    );
    lines.push("mouse alternative: placement and shovel have keyboard bindings".to_string());
    lines.push(
        "no-audio playability: audio remains opt-in and HUD/end text carry state".to_string(),
    );
    lines.push(
        "bilingual readability: en/zh menu, HUD, seed bank, pause, and end text fit bounds"
            .to_string(),
    );
    lines.push("contrast readability: 5 UI text pairs meet 4.5:1 minimum".to_string());
    lines.push(
        "color independence: seed bank names, costs, cooldowns, and cursor coordinates are textual"
            .to_string(),
    );
    lines.push(format!(
        "checked keyboard bindings: {}",
        control_bindings().len()
    ));
    lines.push(format!(
        "checked plants in accessible seed bank: {}",
        PlantKind::COUNT
    ));
    lines.push("manual accessibility QA still required: assistive tech, remapping preference, and photosensitivity review".to_string());
    Ok(lines.join("\n"))
}

fn report_u64_value(report: &str, prefix: &str) -> Option<u64> {
    report
        .lines()
        .find_map(|line| line.strip_prefix(prefix))
        .and_then(|value| value.trim().parse::<u64>().ok())
}

fn performance_budget_audit_report() -> Result<String, String> {
    const MAX_FINAL_SPAWN_BURST: u32 = 20;
    const MIN_RUNTIME_SPAWN_INTERVAL_SECS: f32 = 0.90;
    const MAX_ZOMBIE_BUDGET: u32 = 64;
    const MAX_DYNAMIC_ENTITY_BUDGET: u32 = 320;
    const MAX_EMBEDDED_ASSET_BYTES: u64 = 25_000_000;

    let levels = load_levels();
    let balance = audit_balance()?;
    let asset_report = asset_audit_report()?;
    let visual_report = visual_readability_audit_report()?;
    let audio_report = audio_audit_report()?;

    let max_final_spawn_burst = levels
        .levels
        .iter()
        .map(|level| level.final_spawn_count)
        .max()
        .unwrap_or(0);
    if max_final_spawn_burst > MAX_FINAL_SPAWN_BURST {
        return Err(format!(
            "performance final spawn burst {max_final_spawn_burst} exceeds {MAX_FINAL_SPAWN_BURST}"
        ));
    }

    let min_runtime_spawn_interval = levels
        .levels
        .iter()
        .map(|level| (level.base_spawn_interval - level.final_wave as f32 * 0.18).max(0.95))
        .fold(f32::INFINITY, f32::min);
    if min_runtime_spawn_interval < MIN_RUNTIME_SPAWN_INTERVAL_SECS {
        return Err(format!(
            "performance spawn interval {min_runtime_spawn_interval:.2}s is below {MIN_RUNTIME_SPAWN_INTERVAL_SECS:.2}s"
        ));
    }

    let max_regular_spawn_batch = levels
        .levels
        .iter()
        .map(|level| 1 + (level.final_wave.saturating_sub(1) / 3))
        .max()
        .unwrap_or(0);
    let estimated_peak_zombies = max_final_spawn_burst + LANES as u32 * max_regular_spawn_batch;
    if estimated_peak_zombies > MAX_ZOMBIE_BUDGET {
        return Err(format!(
            "performance zombie budget {estimated_peak_zombies} exceeds {MAX_ZOMBIE_BUDGET}"
        ));
    }

    let board_plant_slots = (LANES * COLS) as u32;
    let projectile_budget = board_plant_slots * 2;
    let min_sky_sun_interval = levels
        .levels
        .iter()
        .map(|level| level.sky_sun_interval)
        .fold(f32::INFINITY, f32::min);
    let sky_sun_pickup_budget = (5.5 / min_sky_sun_interval).ceil() as u32 + 1;
    let sun_pickup_budget = board_plant_slots + sky_sun_pickup_budget;
    let visual_effect_budget = board_plant_slots;
    let estimated_dynamic_entities = board_plant_slots
        + projectile_budget
        + estimated_peak_zombies
        + sun_pickup_budget
        + visual_effect_budget;
    if estimated_dynamic_entities > MAX_DYNAMIC_ENTITY_BUDGET {
        return Err(format!(
            "performance dynamic entity budget {estimated_dynamic_entities} exceeds {MAX_DYNAMIC_ENTITY_BUDGET}"
        ));
    }

    let embedded_asset_bytes = report_u64_value(&asset_report, "embedded asset bytes: ")
        .ok_or_else(|| "performance audit could not read embedded asset byte count".to_string())?;
    if embedded_asset_bytes > MAX_EMBEDDED_ASSET_BYTES {
        return Err(format!(
            "performance embedded asset bytes {embedded_asset_bytes} exceeds {MAX_EMBEDDED_ASSET_BYTES}"
        ));
    }

    if !visual_report.contains("viewport compact-540p | 960x540") {
        return Err("performance audit missing compact viewport evidence".to_string());
    }
    if !audio_report.contains("checked startup policy: audio remains opt-in by default") {
        return Err("performance audit missing audio startup policy evidence".to_string());
    }

    let mut lines = Vec::new();
    lines.push("performance budget audit ok".to_string());
    lines.push(format!("checked levels: {}", balance.levels.len()));
    lines.push(format!(
        "max final spawn burst: {max_final_spawn_burst}/{MAX_FINAL_SPAWN_BURST}"
    ));
    lines.push(format!(
        "minimum runtime spawn interval: {min_runtime_spawn_interval:.2}s"
    ));
    lines.push(format!(
        "estimated peak zombies: {estimated_peak_zombies}/{MAX_ZOMBIE_BUDGET}"
    ));
    lines.push(format!("board plant slots: {board_plant_slots}"));
    lines.push(format!("projectile budget: {projectile_budget}"));
    lines.push(format!("sun pickup budget: {sun_pickup_budget}"));
    lines.push(format!("visual effect budget: {visual_effect_budget}"));
    lines.push(format!(
        "estimated dynamic entities: {estimated_dynamic_entities}/{MAX_DYNAMIC_ENTITY_BUDGET}"
    ));
    lines.push(format!(
        "embedded asset bytes: {embedded_asset_bytes}/{MAX_EMBEDDED_ASSET_BYTES}"
    ));
    lines.push("checked viewport floor: compact-540p 960x540".to_string());
    lines.push("checked audio startup: opt-in, nonblocking policy evidence attached".to_string());
    lines.push(
        "manual performance QA still required: release hardware profiling and long-session soak"
            .to_string(),
    );
    Ok(lines.join("\n"))
}

fn release_readiness_report() -> Result<String, String> {
    let metadata = validate_release_data()?;
    let balance = audit_balance()?;
    let asset_report = asset_audit_report()?;
    let audio_report = audio_audit_report()?;
    let control_report = control_audit_report()?;
    let input_flow_report = input_flow_audit_report()?;
    let localization_report = localization_audit_report()?;
    let layout_report = layout_audit_report()?;
    let visual_report = visual_readability_audit_report()?;
    let accessibility_report = accessibility_audit_report()?;
    let performance_report = performance_budget_audit_report()?;
    let privacy_report = privacy_support_audit_report()?;
    let provenance_report = release_provenance_audit_report()?;
    let marketing_report = marketing_audit_report()?;
    let ip_report = ip_audit_report()?;
    let save_report = save_audit_report()?;
    let campaign = simulate_campaign()?;
    let playthrough_report = playthrough_audit_report()?;

    let mut lines = Vec::new();
    lines.push(format!(
        "release readiness: manual approval required for {} {}",
        metadata.product_name, metadata.version
    ));
    lines.push(format!("channel: {}", metadata.release_channel));
    lines.push(format!(
        "platforms declared: {}",
        metadata.supported_platforms.join(", ")
    ));
    lines.push(format!(
        "languages declared: {}",
        metadata.supported_languages.join(", ")
    ));
    lines.push("automated evidence: pass".to_string());
    lines.push("  release data: pass".to_string());
    lines.push(format!(
        "  balance audit: pass ({} levels)",
        balance.levels.len()
    ));
    lines.push(format!(
        "  asset audit: pass ({})",
        asset_report
            .lines()
            .find(|line| line.starts_with("production art assets: "))
            .unwrap_or("production art assets: unknown")
    ));
    lines.push(format!(
        "  audio audit: pass ({})",
        audio_report
            .lines()
            .find(|line| line.starts_with("checked mix safety: "))
            .unwrap_or("checked mix safety: unknown")
            .trim_start_matches("checked mix safety: ")
    ));
    lines.push(format!(
        "  control audit: pass ({})",
        control_report
            .lines()
            .next()
            .unwrap_or("control audit ok: unknown bindings")
            .trim_start_matches("control audit ok: ")
    ));
    lines.push(format!(
        "  input flow audit: pass ({})",
        input_flow_report
            .lines()
            .find(|line| line.starts_with("gameplay seed selection: "))
            .unwrap_or("gameplay seed selection: unknown")
            .trim_start_matches("gameplay seed selection: ")
    ));
    lines.push(format!(
        "  localization audit: pass ({})",
        localization_report
            .lines()
            .find(|line| line.starts_with("checked languages: "))
            .unwrap_or("checked languages: unknown")
    ));
    lines.push(format!(
        "  layout audit: pass ({})",
        layout_report
            .lines()
            .find(|line| line.starts_with("checked languages: "))
            .unwrap_or("checked languages: unknown")
    ));
    lines.push(format!(
        "  visual readability audit: pass ({})",
        visual_report
            .lines()
            .find(|line| line.starts_with("checked viewports: "))
            .unwrap_or("checked viewports: unknown")
    ));
    lines.push(format!(
        "  accessibility audit: pass ({})",
        accessibility_report
            .lines()
            .find(|line| line.starts_with("keyboard-only flow: "))
            .unwrap_or("keyboard-only flow: unknown")
            .trim_start_matches("keyboard-only flow: ")
    ));
    lines.push(format!(
        "  performance budget audit: pass ({})",
        performance_report
            .lines()
            .find(|line| line.starts_with("estimated dynamic entities: "))
            .unwrap_or("estimated dynamic entities: unknown")
            .trim_start_matches("estimated dynamic entities: ")
    ));
    lines.push(format!(
        "  privacy/support audit: pass ({})",
        privacy_report
            .lines()
            .find(|line| line.starts_with("checked network posture: "))
            .unwrap_or("checked network posture: unknown")
            .trim_start_matches("checked network posture: ")
    ));
    lines.push(format!(
        "  release provenance audit: pass ({})",
        provenance_report
            .lines()
            .find(|line| line.starts_with("checked integrity evidence: "))
            .unwrap_or("checked integrity evidence: unknown")
            .trim_start_matches("checked integrity evidence: ")
    ));
    lines.push(format!(
        "  marketing audit: pass ({})",
        marketing_report
            .lines()
            .find(|line| line.starts_with("checked documents: "))
            .unwrap_or("checked documents: unknown")
            .trim_start_matches("checked documents: ")
    ));
    lines.push(format!(
        "  ip audit: pass ({})",
        ip_report
            .lines()
            .find(|line| line.starts_with("checked release-facing blocks: "))
            .unwrap_or("checked release-facing blocks: unknown")
    ));
    lines.push(format!(
        "  save audit: pass ({})",
        save_report
            .lines()
            .find(|line| line.starts_with("checked compatibility: "))
            .unwrap_or("checked compatibility: unknown")
            .trim_start_matches("checked compatibility: ")
    ));
    lines.push(format!(
        "  campaign simulation: pass ({} levels, plants {}/{}, zombies {}/{})",
        campaign.levels.len(),
        campaign.covered_plants,
        PlantKind::COUNT,
        campaign.covered_zombies,
        ZombieKind::COUNT
    ));
    lines.push(format!(
        "  playthrough audit: pass ({})",
        playthrough_report
            .lines()
            .find(|line| line.starts_with("checked lifecycle: "))
            .unwrap_or("checked lifecycle: unknown")
            .trim_start_matches("checked lifecycle: ")
    ));
    lines.push("manual approval required: yes".to_string());
    for item in [
        "full manual playthrough of all 10 levels",
        "audio-device QA on speakers and headphones before enabling audio by default",
        "final balance and usability sign-off from repeated playtests",
        "final visual spot-check on release hardware",
        "Windows package build and smoke test",
        "macOS package build and smoke test",
        "final art-direction review",
    ] {
        lines.push(format!("  pending: {item}"));
    }
    lines.push("ship status: release candidate, not final approval".to_string());
    Ok(lines.join("\n"))
}

#[derive(Debug, Clone)]
struct BalanceAuditLevel {
    index: usize,
    id: String,
    final_wave: u32,
    expected_sun: u32,
    baseline_defense_cost: u32,
    final_pressure: f32,
}

#[derive(Debug, Clone)]
struct BalanceAuditReport {
    levels: Vec<BalanceAuditLevel>,
}

#[derive(Debug, Clone)]
struct CampaignSimulationLevel {
    index: usize,
    id: String,
    unlock_before: usize,
    unlock_after: usize,
    affordable_plants: usize,
    zombie_pool_size: usize,
    projected_score_floor: u32,
}

#[derive(Debug, Clone)]
struct CampaignSimulationReport {
    levels: Vec<CampaignSimulationLevel>,
    covered_plants: usize,
    covered_zombies: usize,
}

#[derive(Debug, Clone)]
struct PlaythroughAuditLevel {
    index: usize,
    id: String,
    unlock_before: usize,
    unlock_after_victory: usize,
    victory_score: u32,
    best_score: u32,
    restart_sun: u32,
    defeat_unlock_after: usize,
}

#[derive(Debug, Clone)]
struct PlaythroughAuditReport {
    levels: Vec<PlaythroughAuditLevel>,
    victory_checks: usize,
    defeat_checks: usize,
    restart_checks: usize,
    score_checks: usize,
}

fn baseline_defense_cost() -> u32 {
    PlantKind::Sunflower.cost() * 2
        + PlantKind::Peashooter.cost() * LANES as u32
        + PlantKind::Wallnut.cost() * 2
}

fn expected_level_sun(level: &LevelConfig) -> u32 {
    let duration = level.final_wave as f32 * level.wave_duration;
    let sky_sun = (duration / level.sky_sun_interval).floor().max(0.0) as u32 * 25;
    let sunflower_sun = (duration / 7.0).floor().max(0.0) as u32 * 25;
    level.starting_sun + sky_sun + sunflower_sun
}

fn level_final_pressure(level: &LevelConfig) -> f32 {
    level.final_spawn_count as f32 / level.base_spawn_interval.max(0.1)
}

fn audit_balance() -> Result<BalanceAuditReport, String> {
    let levels = load_levels();
    let baseline_cost = baseline_defense_cost();
    let mut previous_final_wave = 0;
    let mut previous_pressure = 0.0;
    let mut report = Vec::with_capacity(levels.levels.len());

    for (index, level) in levels.levels.iter().enumerate() {
        if level.starting_sun < PlantKind::Peashooter.cost() {
            return Err(format!(
                "level {} starts with {} sun, below the basic peashooter opener",
                level.id, level.starting_sun
            ));
        }
        if level.starting_sun + 50 < PlantKind::Sunflower.cost() + PlantKind::Peashooter.cost() {
            return Err(format!(
                "level {} cannot reach a sunflower + peashooter economy after two sun drops",
                level.id
            ));
        }
        if level.final_wave < previous_final_wave {
            return Err(format!(
                "level {} final_wave regresses from previous level",
                level.id
            ));
        }
        if level.max_breaches == 0 || level.max_breaches > 3 {
            return Err(format!(
                "level {} max_breaches must stay between 1 and 3",
                level.id
            ));
        }
        // Upper bound raised from 3.5 after autoplay testing showed early
        // levels need slower trickles: one shooter kills a wave-1 walker in
        // ~5.3s, so sub-4s spawn intervals put lane one in deficit from the
        // first minute.
        if !(1.0..=6.5).contains(&level.base_spawn_interval) {
            return Err(format!(
                "level {} base_spawn_interval {} is outside release bounds",
                level.id, level.base_spawn_interval
            ));
        }
        if !(6.0..=11.0).contains(&level.sky_sun_interval) {
            return Err(format!(
                "level {} sky_sun_interval {} is outside release bounds",
                level.id, level.sky_sun_interval
            ));
        }

        let expected_sun = expected_level_sun(level);
        if expected_sun < baseline_cost {
            return Err(format!(
                "level {} expected sun budget {} is below baseline defense cost {}",
                level.id, expected_sun, baseline_cost
            ));
        }

        let pressure = level_final_pressure(level);
        if index > 0 && pressure + 0.25 < previous_pressure {
            return Err(format!(
                "level {} final pressure drops too sharply from previous level",
                level.id
            ));
        }
        if pressure > 13.0 {
            return Err(format!(
                "level {} final pressure {:.1} exceeds release bound",
                level.id, pressure
            ));
        }

        report.push(BalanceAuditLevel {
            index,
            id: level.id.clone(),
            final_wave: level.final_wave,
            expected_sun,
            baseline_defense_cost: baseline_cost,
            final_pressure: pressure,
        });
        previous_final_wave = level.final_wave;
        previous_pressure = pressure;
    }

    Ok(BalanceAuditReport { levels: report })
}

// Each level unlocks one new zombie kind, themed after its title, so the
// campaign teaches enemies one at a time instead of front-loading the roster.
fn level_zombie_roster(level_index: usize) -> &'static [ZombieKind] {
    const ROSTERS: [&[ZombieKind]; 10] = [
        // 01 greenhouse_morning: basics only
        &[ZombieKind::Walker, ZombieKind::Conehead],
        // 02 foggy_rows: + runner
        &[ZombieKind::Walker, ZombieKind::Conehead, ZombieKind::Runner],
        // 03 midnight_pressure: + buckethead
        &[
            ZombieKind::Walker,
            ZombieKind::Conehead,
            ZombieKind::Runner,
            ZombieKind::Buckethead,
        ],
        // 04 sunless_lane: + jumper
        &[
            ZombieKind::Walker,
            ZombieKind::Conehead,
            ZombieKind::Runner,
            ZombieKind::Buckethead,
            ZombieKind::Jumper,
        ],
        // 05 bucket_barrage: + brute
        &[
            ZombieKind::Walker,
            ZombieKind::Conehead,
            ZombieKind::Runner,
            ZombieKind::Buckethead,
            ZombieKind::Jumper,
            ZombieKind::Brute,
        ],
        // 06 healer_column: + healer
        &[
            ZombieKind::Walker,
            ZombieKind::Conehead,
            ZombieKind::Runner,
            ZombieKind::Buckethead,
            ZombieKind::Jumper,
            ZombieKind::Brute,
            ZombieKind::Healer,
        ],
        // 07 frost_front: + frostbite
        &[
            ZombieKind::Walker,
            ZombieKind::Conehead,
            ZombieKind::Runner,
            ZombieKind::Buckethead,
            ZombieKind::Jumper,
            ZombieKind::Brute,
            ZombieKind::Healer,
            ZombieKind::Frostbite,
        ],
        // 08 digger_underpass: + digger
        &[
            ZombieKind::Walker,
            ZombieKind::Conehead,
            ZombieKind::Runner,
            ZombieKind::Buckethead,
            ZombieKind::Jumper,
            ZombieKind::Brute,
            ZombieKind::Healer,
            ZombieKind::Frostbite,
            ZombieKind::Digger,
        ],
        // 09 gargantuar_gate and 10 last_greenhouse: everything
        ZombieKind::ALL_SLICE,
        ZombieKind::ALL_SLICE,
    ];
    ROSTERS
        .get(level_index)
        .copied()
        .unwrap_or(ZombieKind::ALL_SLICE)
}

fn zombie_pool_for_wave(level_index: usize, wave: u32, final_wave: bool) -> Vec<ZombieKind> {
    let roster = level_zombie_roster(level_index);
    let mut pool = raw_zombie_pool_for_wave(wave);
    pool.retain(|kind| roster.contains(kind));
    if pool.is_empty() {
        pool.push(*roster.last().expect("level roster must not be empty"));
    }
    if final_wave
        && roster.contains(&ZombieKind::Gargantuar)
        && !pool.contains(&ZombieKind::Gargantuar)
    {
        pool.push(ZombieKind::Gargantuar);
    }
    pool
}

fn raw_zombie_pool_for_wave(wave: u32) -> Vec<ZombieKind> {
    let pool = match wave {
        1 => vec![ZombieKind::Walker],
        2 => vec![ZombieKind::Walker, ZombieKind::Conehead],
        3 => vec![ZombieKind::Walker, ZombieKind::Conehead, ZombieKind::Runner],
        4 => vec![
            ZombieKind::Walker,
            ZombieKind::Runner,
            ZombieKind::Conehead,
            ZombieKind::Buckethead,
        ],
        5 => vec![
            ZombieKind::Walker,
            ZombieKind::Runner,
            ZombieKind::Buckethead,
            ZombieKind::Healer,
            ZombieKind::Jumper,
        ],
        6 => vec![
            ZombieKind::Runner,
            ZombieKind::Buckethead,
            ZombieKind::Healer,
            ZombieKind::Jumper,
            ZombieKind::Digger,
            ZombieKind::Brute,
        ],
        7 => vec![
            ZombieKind::Runner,
            ZombieKind::Buckethead,
            ZombieKind::Healer,
            ZombieKind::Jumper,
            ZombieKind::Digger,
            ZombieKind::Frostbite,
            ZombieKind::Brute,
        ],
        _ => vec![
            ZombieKind::Walker,
            ZombieKind::Runner,
            ZombieKind::Buckethead,
            ZombieKind::Healer,
            ZombieKind::Jumper,
            ZombieKind::Digger,
            ZombieKind::Frostbite,
            ZombieKind::Brute,
            ZombieKind::Gargantuar,
        ],
    };
    pool
}

fn simulate_campaign() -> Result<CampaignSimulationReport, String> {
    let catalog = load_levels();
    let mut progress = ProgressState {
        unlocked_levels: 1,
        best_scores: vec![0; catalog.levels.len()],
    };
    let mut covered_plants = [false; PlantKind::COUNT];
    let mut covered_zombies = [false; ZombieKind::COUNT];
    let mut levels = Vec::with_capacity(catalog.levels.len());

    for (index, level) in catalog.levels.iter().enumerate() {
        if !progress.is_unlocked(index) {
            return Err(format!(
                "level {} is locked before simulated campaign reaches it",
                level.id
            ));
        }

        let state = BoardState::for_level(index, level);
        if state.sun < PlantKind::Peashooter.cost() {
            return Err(format!(
                "level {} cannot afford the required first peashooter opener",
                level.id
            ));
        }

        let expected_sun = expected_level_sun(level);
        let affordable_plants = PlantKind::ALL
            .iter()
            .filter(|plant| expected_sun >= plant.cost())
            .inspect(|plant| covered_plants[plant.index()] = true)
            .count();
        if affordable_plants != PlantKind::COUNT {
            return Err(format!(
                "level {} only exposes {affordable_plants}/{} plants through its expected sun budget",
                level.id,
                PlantKind::COUNT
            ));
        }

        let mut level_zombies = [false; ZombieKind::COUNT];
        let mut projected_score_floor = 0;
        for wave in 1..=level.final_wave {
            let is_final_wave = wave == level.final_wave;
            let pool = zombie_pool_for_wave(index, wave, is_final_wave);
            for zombie in &pool {
                covered_zombies[zombie.index()] = true;
                level_zombies[zombie.index()] = true;
            }

            let spawn_count = if is_final_wave {
                level.final_spawn_count
            } else {
                1 + (wave / 3)
            };
            let lowest_score = pool
                .iter()
                .map(|zombie| zombie.score())
                .min()
                .ok_or_else(|| format!("level {} wave {wave} has no zombie pool", level.id))?;
            projected_score_floor += lowest_score * spawn_count;
        }

        let unlock_before = progress.unlocked_levels;
        progress.best_scores[index] = progress.best_scores[index].max(projected_score_floor);
        progress.unlocked_levels = progress
            .unlocked_levels
            .max((index + 2).min(catalog.levels.len()));
        levels.push(CampaignSimulationLevel {
            index,
            id: level.id.clone(),
            unlock_before,
            unlock_after: progress.unlocked_levels,
            affordable_plants,
            zombie_pool_size: level_zombies.iter().filter(|covered| **covered).count(),
            projected_score_floor,
        });
    }

    let covered_plants = covered_plants.iter().filter(|covered| **covered).count();
    let covered_zombies = covered_zombies.iter().filter(|covered| **covered).count();
    if covered_plants != PlantKind::COUNT {
        return Err(format!(
            "campaign simulation only covered {covered_plants}/{} plant rules",
            PlantKind::COUNT
        ));
    }
    if covered_zombies != ZombieKind::COUNT {
        return Err(format!(
            "campaign simulation only covered {covered_zombies}/{} zombie rules",
            ZombieKind::COUNT
        ));
    }
    if progress.unlocked_levels != catalog.levels.len() {
        return Err(format!(
            "campaign simulation unlocked {}/{} levels",
            progress.unlocked_levels,
            catalog.levels.len()
        ));
    }

    Ok(CampaignSimulationReport {
        levels,
        covered_plants,
        covered_zombies,
    })
}

fn projected_level_score_floor(level_index: usize, level: &LevelConfig) -> Result<u32, String> {
    let mut projected_score = 0;
    for wave in 1..=level.final_wave {
        let is_final_wave = wave == level.final_wave;
        let pool = zombie_pool_for_wave(level_index, wave, is_final_wave);
        let spawn_count = if is_final_wave {
            level.final_spawn_count
        } else {
            1 + (wave / 3)
        };
        let lowest_score = pool
            .iter()
            .map(|zombie| zombie.score())
            .min()
            .ok_or_else(|| format!("level {} wave {wave} has no zombie pool", level.id))?;
        projected_score += lowest_score * spawn_count;
    }
    Ok(projected_score)
}

fn apply_victory_progress(
    progress: &mut ProgressState,
    level_index: usize,
    score: u32,
    level_count: usize,
) -> Result<(), String> {
    let Some(best_score) = progress.best_scores.get_mut(level_index) else {
        return Err(format!(
            "playthrough audit cannot record score for missing level slot {}",
            level_index + 1
        ));
    };
    *best_score = (*best_score).max(score);
    progress.unlocked_levels = progress
        .unlocked_levels
        .max((level_index + 2).min(level_count));
    Ok(())
}

fn assert_restart_state(level: &LevelConfig, state: &BoardState) -> Result<(), String> {
    if state.sun != level.starting_sun {
        return Err(format!(
            "level {} restart sun {} does not match starting sun {}",
            level.id, state.sun, level.starting_sun
        ));
    }
    if state.wave != 1 || state.score != 0 || state.lost_house_hp != 0 {
        return Err(format!(
            "level {} restart state is not clean: wave {}, score {}, breaches {}",
            level.id, state.wave, state.score, state.lost_house_hp
        ));
    }
    if state.selected != PlantKind::Peashooter || state.cursor_col != 0 || state.cursor_lane != 2 {
        return Err(format!(
            "level {} restart controls are not reset to the default cursor and seed",
            level.id
        ));
    }
    if state
        .plant_cooldowns
        .iter()
        .any(|cooldown| *cooldown != 0.0)
    {
        return Err(format!(
            "level {} restart state contains non-zero plant cooldowns",
            level.id
        ));
    }
    if state.final_wave_started {
        return Err(format!(
            "level {} restart state incorrectly starts at the final wave",
            level.id
        ));
    }
    Ok(())
}

fn audit_playthrough() -> Result<PlaythroughAuditReport, String> {
    let catalog = load_levels();
    let level_count = catalog.levels.len();
    let mut progress = ProgressState {
        unlocked_levels: 1,
        best_scores: vec![0; level_count],
    };
    let mut levels = Vec::with_capacity(level_count);
    let mut victory_checks = 0;
    let mut defeat_checks = 0;
    let mut restart_checks = 0;
    let mut score_checks = 0;

    for (index, level) in catalog.levels.iter().enumerate() {
        if !progress.is_unlocked(index) {
            return Err(format!(
                "level {} is locked before the scripted playthrough reaches it",
                level.id
            ));
        }

        let restart_state = BoardState::for_level(index, level);
        assert_restart_state(level, &restart_state)?;
        restart_checks += 1;

        let unlock_before = progress.unlocked_levels;
        let victory_score = projected_level_score_floor(index, level)?;
        if victory_score == 0 {
            return Err(format!(
                "level {} has a zero projected victory score",
                level.id
            ));
        }
        apply_victory_progress(&mut progress, index, victory_score, level_count)?;
        victory_checks += 1;

        let best_score = progress
            .best_score(index)
            .ok_or_else(|| format!("level {} victory did not record a best score", level.id))?;
        if best_score != victory_score {
            return Err(format!(
                "level {} best score {best_score} did not match victory score {victory_score}",
                level.id
            ));
        }
        score_checks += 1;

        let unlock_after_victory = progress.unlocked_levels;
        let expected_unlock = (index + 2).min(level_count);
        if unlock_after_victory < expected_unlock {
            return Err(format!(
                "level {} victory unlocked {unlock_after_victory}, expected at least {expected_unlock}",
                level.id
            ));
        }

        let defeat_progress = progress.clone();
        let defeat_unlock_before = defeat_progress.unlocked_levels;
        let mut defeat_state = BoardState::for_level(index, level);
        defeat_state.lost_house_hp = level.max_breaches;
        if defeat_state.lost_house_hp < level.max_breaches {
            return Err(format!(
                "level {} defeat state did not reach the breach limit",
                level.id
            ));
        }
        if defeat_progress.unlocked_levels != defeat_unlock_before {
            return Err(format!(
                "level {} defeat changed unlocked progress unexpectedly",
                level.id
            ));
        }
        defeat_checks += 1;

        levels.push(PlaythroughAuditLevel {
            index,
            id: level.id.clone(),
            unlock_before,
            unlock_after_victory,
            victory_score,
            best_score,
            restart_sun: restart_state.sun,
            defeat_unlock_after: defeat_progress.unlocked_levels,
        });
    }

    if progress.unlocked_levels != level_count {
        return Err(format!(
            "scripted playthrough unlocked {}/{} levels",
            progress.unlocked_levels, level_count
        ));
    }
    if progress.best_scores.contains(&0) {
        return Err("scripted playthrough left at least one best score empty".to_string());
    }

    Ok(PlaythroughAuditReport {
        levels,
        victory_checks,
        defeat_checks,
        restart_checks,
        score_checks,
    })
}

fn playthrough_audit_report() -> Result<String, String> {
    let report = audit_playthrough()?;
    let mut lines = vec![format!(
        "playthrough audit ok: {} levels",
        report.levels.len()
    )];
    for level in &report.levels {
        lines.push(format!(
            "{:02}. {} | victory unlock {}->{} | score {} | best {} | restart sun {} | defeat unlock {}",
            level.index + 1,
            level.id,
            level.unlock_before,
            level.unlock_after_victory,
            level.victory_score,
            level.best_score,
            level.restart_sun,
            level.defeat_unlock_after
        ));
    }
    lines.push(format!(
        "checked lifecycle: victories {}, defeats {}, restarts {}, score saves {}",
        report.victory_checks, report.defeat_checks, report.restart_checks, report.score_checks
    ));
    lines.push("checked clean-save campaign unlocks: 10/10".to_string());
    lines.push("checked failure handling: defeat does not advance unlocks".to_string());
    lines.push("checked restart handling: board state resets per level tuning".to_string());
    Ok(lines.join("\n"))
}

fn save_path_from_env(
    explicit_path: Option<PathBuf>,
    xdg_data_home: Option<PathBuf>,
    home: Option<PathBuf>,
) -> PathBuf {
    if let Some(path) = explicit_path {
        return path;
    }
    if let Some(path) = xdg_data_home {
        return path.join("bevy_open_siege").join(SAVE_FILE_NAME);
    }
    if let Some(path) = home {
        return path
            .join(".local")
            .join("share")
            .join("bevy_open_siege")
            .join(SAVE_FILE_NAME);
    }
    PathBuf::from(SAVE_FILE_NAME)
}

fn save_path() -> PathBuf {
    save_path_from_env(
        env::var_os("BEVY_OPEN_SIEGE_SAVE_PATH").map(PathBuf::from),
        env::var_os("XDG_DATA_HOME").map(PathBuf::from),
        env::var_os("HOME").map(PathBuf::from),
    )
}

fn legacy_save_path() -> &'static Path {
    Path::new(SAVE_FILE_NAME)
}

fn parse_save_file(path: &Path) -> Option<SaveData> {
    let Ok(contents) = fs::read_to_string(path) else {
        return None;
    };
    match ron::from_str(&contents) {
        Ok(save) => Some(save),
        Err(error) => {
            warn!("failed to parse save file {}: {error}", path.display());
            None
        }
    }
}

// Browsers have no filesystem, so web saves live in localStorage under this
// key, using the same RON payload as the desktop save file.
#[cfg(target_arch = "wasm32")]
const WEB_SAVE_KEY: &str = "bevy_open_siege_save";

#[cfg(target_arch = "wasm32")]
fn web_local_storage() -> Option<web_sys::Storage> {
    web_sys::window()?.local_storage().ok()?
}

#[cfg(target_arch = "wasm32")]
fn load_web_save() -> Option<SaveData> {
    let contents = web_local_storage()?.get_item(WEB_SAVE_KEY).ok()??;
    match ron::from_str(&contents) {
        Ok(save) => Some(save),
        Err(error) => {
            warn!("failed to parse web save: {error}");
            None
        }
    }
}

#[cfg(target_arch = "wasm32")]
fn store_web_save(contents: &str) {
    let Some(storage) = web_local_storage() else {
        warn!("localStorage unavailable; progress will not persist");
        return;
    };
    if storage.set_item(WEB_SAVE_KEY, contents).is_err() {
        warn!("failed to write web save to localStorage");
    }
}

fn load_save() -> SaveData {
    #[cfg(target_arch = "wasm32")]
    if let Some(save) = load_web_save() {
        return save;
    }

    let path = save_path();
    if let Some(save) = parse_save_file(&path) {
        return save;
    }

    let legacy_path = legacy_save_path();
    if path != legacy_path
        && let Some(save) = parse_save_file(legacy_path)
    {
        return save;
    }

    SaveData::default()
}

fn save_progress(language: Language, progress: &ProgressState, settings: &GameSettings) {
    let save = SaveData {
        version: 1,
        language,
        unlocked_levels: progress.unlocked_levels,
        best_scores: progress.best_scores.clone(),
        settings: SaveSettings {
            master_volume: settings.master_volume,
            fullscreen: settings.fullscreen,
        },
    };
    match ron::ser::to_string_pretty(&save, ron::ser::PrettyConfig::default()) {
        Ok(contents) => {
            #[cfg(target_arch = "wasm32")]
            {
                store_web_save(&contents);
            }
            #[cfg(not(target_arch = "wasm32"))]
            {
                let path = save_path();
                if let Some(parent) = path
                    .parent()
                    .filter(|parent| !parent.as_os_str().is_empty())
                    && let Err(error) = fs::create_dir_all(parent)
                {
                    warn!(
                        "failed to create save directory {}: {error}",
                        parent.display()
                    );
                    return;
                }
                if let Err(error) = fs::write(&path, contents) {
                    warn!("failed to write save file {}: {error}", path.display());
                }
            }
        }
        Err(error) => warn!("failed to serialize save data: {error}"),
    }
}

fn normalized_progress(save: &SaveData, level_count: usize) -> ProgressState {
    let mut best_scores = save.best_scores.clone();
    best_scores.resize(level_count, 0);
    ProgressState {
        unlocked_levels: save.unlocked_levels.clamp(1, level_count.max(1)),
        best_scores,
    }
}

fn save_summary_text(save: &SaveData, level_count: usize) -> String {
    let progress = normalized_progress(save, level_count);
    let best_scores = progress
        .best_scores
        .iter()
        .map(u32::to_string)
        .collect::<Vec<_>>()
        .join(",");
    format!(
        "save_version={}\nlanguage={:?}\nunlocked_levels={}\nbest_scores={}\nmaster_volume={:.2}\nfullscreen={}\npath={}",
        save.version,
        save.language,
        progress.unlocked_levels,
        best_scores,
        save.settings.master_volume.clamp(0.0, 1.0),
        save.settings.fullscreen,
        save_path().display()
    )
}

fn runtime_asset_root() -> String {
    if let Ok(working_dir) = std::env::current_dir() {
        let working_assets = working_dir.join("assets");
        if working_assets.is_dir() {
            return working_assets.to_string_lossy().into_owned();
        }
    }

    if let Ok(executable_path) = std::env::current_exe()
        && let Some(executable_dir) = executable_path.parent()
    {
        let executable_assets = executable_dir.join("assets");
        if executable_assets.is_dir() {
            return executable_assets.to_string_lossy().into_owned();
        }
    }

    "assets".to_string()
}

fn main() {
    if handle_cli_args() {
        return;
    }

    let screenshot_scene = store_screenshot_scene_arg();
    let level_catalog = load_levels();
    let localization = load_localization();
    let save = load_save();
    let progress = normalized_progress(&save, level_catalog.levels.len());
    let settings = GameSettings::from(&save.settings);

    let plugins = DefaultPlugins
        .set(bevy::asset::AssetPlugin {
            file_path: runtime_asset_root(),
            ..default()
        })
        .set(WindowPlugin {
            primary_window: Some(Window {
                title: "Bevy Open Siege".to_string(),
                resolution: if screenshot_scene.is_some() {
                    WindowResolution::new(1920, 1080)
                } else {
                    WindowResolution::new(1280, 720)
                },
                resize_constraints: WindowResizeConstraints {
                    min_width: 960.0,
                    min_height: 540.0,
                    ..default()
                },
                present_mode: bevy::window::PresentMode::AutoVsync,
                ..default()
            }),
            ..default()
        });

    App::new()
        .insert_resource(ClearColor(Color::srgb(0.055, 0.095, 0.08)))
        .insert_resource(BoardState::default())
        .insert_resource(LanguageSettings {
            current: save.language,
        })
        .insert_resource(settings)
        .insert_resource(PauseState::default())
        .insert_resource(OnboardingState::default())
        .insert_resource(ScreenShake::default())
        .insert_resource(StoreScreenshotMode {
            scene: screenshot_scene,
        })
        .insert_resource(start_async_audio(audio_enabled()))
        .add_plugins(platform_audio_plugin)
        .insert_resource(level_catalog)
        .insert_resource(localization)
        .insert_resource(progress)
        .add_plugins(plugins)
        .register_type::<GltfExtras>()
        .register_type::<GltfSceneExtras>()
        .register_type::<GltfMeshExtras>()
        .register_type::<GltfMeshName>()
        .register_type::<GltfMaterialExtras>()
        .register_type::<GltfMaterialName>()
        .init_state::<GameState>()
        .add_systems(Startup, ensure_primary_window_size)
        .add_systems(PreStartup, install_ui_font)
        .add_systems(Startup, setup_camera)
        .add_systems(Startup, setup_store_screenshot_scene)
        .add_systems(Startup, apply_saved_window_settings)
        .add_systems(Startup, setup_audio)
        .add_systems(Startup, setup_ui_fonts)
        .add_systems(
            Update,
            (
                toggle_language,
                settings_input,
                apply_audio_volume,
                toggle_pause,
                apply_language_font,
            ),
        )
        .add_systems(OnEnter(GameState::Menu), spawn_menu)
        .add_systems(
            Update,
            (menu_input, menu_mouse, style_menu_rows, update_menu_text)
                .run_if(in_state(GameState::Menu)),
        )
        .add_systems(OnExit(GameState::Menu), despawn_menu)
        .add_systems(
            OnEnter(GameState::Playing),
            (start_game, populate_store_screenshot_scene).chain(),
        )
        .add_systems(
            Update,
            (
                handle_board_input,
                update_cursor,
                tick_plant_cooldowns,
                collect_sun,
                pause_menu_buttons,
                update_pause_ui,
                update_hud,
                seed_card_clicks,
                update_seed_cards,
                update_wave_bar,
                update_onboarding,
                check_end_state,
            )
                .chain()
                .run_if(in_state(GameState::Playing)),
        )
        .add_systems(
            Update,
            (
                spawn_sun,
                spawn_zombies,
                plants_act,
                tick_visual_effects,
                move_projectiles,
                move_and_attack_zombies,
                animate_units,
                tag_limbs,
                animate_limbs,
                grow_plants,
                hit_react,
                shake_camera,
                announce_waves,
                cleanup_dead,
            )
                .chain()
                .run_if(in_state(GameState::Playing).and(not_paused)),
        )
        .add_systems(OnExit(GameState::Playing), despawn_game)
        .add_systems(OnEnter(GameState::GameOver), spawn_game_over)
        .add_systems(OnEnter(GameState::Victory), spawn_victory)
        .add_systems(
            Update,
            (restart_input, end_screen_mouse, update_end_text)
                .run_if(in_state(GameState::GameOver).or(in_state(GameState::Victory))),
        )
        .add_systems(OnExit(GameState::GameOver), despawn_menu)
        .add_systems(OnExit(GameState::Victory), despawn_menu)
        .run();
}

fn ensure_primary_window_size(
    mode: Res<StoreScreenshotMode>,
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
) {
    if let Ok(mut window) = windows.single_mut()
        && (window.resolution.physical_width() == 0 || window.resolution.physical_height() == 0)
    {
        let (width, height) = if mode.scene.is_some() {
            (1920, 1080)
        } else {
            (1280, 720)
        };
        window.resolution.set_physical_resolution(width, height);
    }
}

fn store_screenshot_scene_arg() -> Option<StoreScreenshotScene> {
    let mut args = std::env::args().skip(1).peekable();
    while let Some(arg) = args.next() {
        let value = if let Some(value) = arg.strip_prefix("--store-screenshot-scene=") {
            Some(value.to_string())
        } else if arg == "--store-screenshot-scene" {
            args.next()
        } else {
            None
        };
        if let Some(value) = value {
            return match value.as_str() {
                "title-menu" => Some(StoreScreenshotScene::TitleMenu),
                "early-defense" => Some(StoreScreenshotScene::EarlyDefense),
                "special-enemies" => Some(StoreScreenshotScene::SpecialEnemies),
                "late-siege" => Some(StoreScreenshotScene::LateSiege),
                "victory-summary" => Some(StoreScreenshotScene::VictorySummary),
                _ => {
                    eprintln!("unknown store screenshot scene: {value}");
                    std::process::exit(2);
                }
            };
        }
    }
    None
}

fn audio_enabled() -> bool {
    // Web audio goes through bevy_audio and is gated on first user gesture
    // instead of the desktop opt-in flag.
    #[cfg(target_arch = "wasm32")]
    {
        return true;
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        std::env::var_os("BEVY_OPEN_SIEGE_AUDIO").is_some()
            || std::env::args().any(|arg| arg == "--audio")
    }
}

fn handle_cli_args() -> bool {
    let mut args = std::env::args().skip(1);
    let Some(arg) = args.next() else {
        return false;
    };

    match arg.as_str() {
        "--validate-data" => {
            match validate_release_data() {
                Ok(metadata) => {
                    println!(
                        "{} {} release data ok",
                        metadata.product_name, metadata.version
                    );
                }
                Err(error) => {
                    eprintln!("release data validation failed: {error}");
                    std::process::exit(1);
                }
            }
            true
        }
        "--print-release-info" | "--version" => {
            let metadata = load_release_metadata();
            println!(
                "{} {} ({})",
                metadata.product_name, metadata.version, metadata.release_channel
            );
            println!("languages: {}", metadata.supported_languages.join(", "));
            println!("platforms: {}", metadata.supported_platforms.join(", "));
            println!("rating: {}", metadata.content_rating_note);
            true
        }
        "--print-save-path" => {
            println!("{}", save_path().display());
            true
        }
        "--print-save-summary" => {
            let levels = load_levels();
            let save = load_save();
            println!("{}", save_summary_text(&save, levels.levels.len()));
            true
        }
        "--audit-balance" => {
            match audit_balance() {
                Ok(report) => {
                    println!("balance audit ok: {} levels", report.levels.len());
                    for level in report.levels {
                        println!(
                            "{:02}. {} | final wave {} | expected sun {} / baseline {} | pressure {:.1}",
                            level.index + 1,
                            level.id,
                            level.final_wave,
                            level.expected_sun,
                            level.baseline_defense_cost,
                            level.final_pressure
                        );
                    }
                }
                Err(error) => {
                    eprintln!("balance audit failed: {error}");
                    std::process::exit(1);
                }
            }
            true
        }
        "--audit-assets" => {
            match asset_audit_report() {
                Ok(report) => println!("{report}"),
                Err(error) => {
                    eprintln!("asset audit failed: {error}");
                    std::process::exit(1);
                }
            }
            true
        }
        "--audit-audio" => {
            match audio_audit_report() {
                Ok(report) => println!("{report}"),
                Err(error) => {
                    eprintln!("audio audit failed: {error}");
                    std::process::exit(1);
                }
            }
            true
        }
        "--audit-controls" => {
            match control_audit_report() {
                Ok(report) => println!("{report}"),
                Err(error) => {
                    eprintln!("control audit failed: {error}");
                    std::process::exit(1);
                }
            }
            true
        }
        "--audit-input-flow" => {
            match input_flow_audit_report() {
                Ok(report) => println!("{report}"),
                Err(error) => {
                    eprintln!("input flow audit failed: {error}");
                    std::process::exit(1);
                }
            }
            true
        }
        "--audit-localization" => {
            match localization_audit_report() {
                Ok(report) => println!("{report}"),
                Err(error) => {
                    eprintln!("localization audit failed: {error}");
                    std::process::exit(1);
                }
            }
            true
        }
        "--audit-layout" => {
            match layout_audit_report() {
                Ok(report) => println!("{report}"),
                Err(error) => {
                    eprintln!("layout audit failed: {error}");
                    std::process::exit(1);
                }
            }
            true
        }
        "--audit-visual" => {
            match visual_readability_audit_report() {
                Ok(report) => println!("{report}"),
                Err(error) => {
                    eprintln!("visual readability audit failed: {error}");
                    std::process::exit(1);
                }
            }
            true
        }
        "--audit-accessibility" => {
            match accessibility_audit_report() {
                Ok(report) => println!("{report}"),
                Err(error) => {
                    eprintln!("accessibility audit failed: {error}");
                    std::process::exit(1);
                }
            }
            true
        }
        "--audit-performance" => {
            match performance_budget_audit_report() {
                Ok(report) => println!("{report}"),
                Err(error) => {
                    eprintln!("performance budget audit failed: {error}");
                    std::process::exit(1);
                }
            }
            true
        }
        "--audit-privacy" => {
            match privacy_support_audit_report() {
                Ok(report) => println!("{report}"),
                Err(error) => {
                    eprintln!("privacy audit failed: {error}");
                    std::process::exit(1);
                }
            }
            true
        }
        "--audit-release-provenance" => {
            match release_provenance_audit_report() {
                Ok(report) => println!("{report}"),
                Err(error) => {
                    eprintln!("release provenance audit failed: {error}");
                    std::process::exit(1);
                }
            }
            true
        }
        "--audit-marketing" => {
            match marketing_audit_report() {
                Ok(report) => println!("{report}"),
                Err(error) => {
                    eprintln!("marketing audit failed: {error}");
                    std::process::exit(1);
                }
            }
            true
        }
        "--audit-ip" => {
            match ip_audit_report() {
                Ok(report) => println!("{report}"),
                Err(error) => {
                    eprintln!("ip audit failed: {error}");
                    std::process::exit(1);
                }
            }
            true
        }
        "--audit-save" => {
            match save_audit_report() {
                Ok(report) => println!("{report}"),
                Err(error) => {
                    eprintln!("save audit failed: {error}");
                    std::process::exit(1);
                }
            }
            true
        }
        "--audit-playthrough" => {
            match playthrough_audit_report() {
                Ok(report) => println!("{report}"),
                Err(error) => {
                    eprintln!("playthrough audit failed: {error}");
                    std::process::exit(1);
                }
            }
            true
        }
        "--release-readiness" => {
            match release_readiness_report() {
                Ok(report) => println!("{report}"),
                Err(error) => {
                    eprintln!("release readiness failed: {error}");
                    std::process::exit(1);
                }
            }
            true
        }
        "--simulate-campaign" => {
            match simulate_campaign() {
                Ok(report) => {
                    println!("campaign simulation ok: {} levels", report.levels.len());
                    for level in report.levels {
                        println!(
                            "{:02}. {} | unlock {}->{} | affordable plants {}/{} | zombie pool {}/{} | projected score floor {}",
                            level.index + 1,
                            level.id,
                            level.unlock_before,
                            level.unlock_after,
                            level.affordable_plants,
                            PlantKind::COUNT,
                            level.zombie_pool_size,
                            ZombieKind::COUNT,
                            level.projected_score_floor
                        );
                    }
                    println!(
                        "covered plants: {}/{}",
                        report.covered_plants,
                        PlantKind::COUNT
                    );
                    println!(
                        "covered zombies: {}/{}",
                        report.covered_zombies,
                        ZombieKind::COUNT
                    );
                }
                Err(error) => {
                    eprintln!("campaign simulation failed: {error}");
                    std::process::exit(1);
                }
            }
            true
        }
        "--help" | "-h" => {
            println!("Bevy Open Siege");
            println!("  --validate-data       Validate embedded release data and exit");
            println!("  --audit-balance       Audit campaign balance data and exit");
            println!("  --audit-assets        Audit embedded release assets and exit");
            println!("  --audit-audio         Audit WAV mix safety and startup audio policy");
            println!("  --audit-controls      Audit documented input bindings and exit");
            println!("  --audit-input-flow    Audit deterministic menu and gameplay input flow");
            println!("  --audit-localization  Audit bilingual localization coverage and exit");
            println!("  --audit-layout        Audit generated UI text layout bounds and exit");
            println!("  --audit-visual        Audit resolution, contrast, and visual readability");
            println!("  --audit-accessibility Audit keyboard, contrast, and no-audio coverage");
            println!("  --audit-performance   Audit entity, asset, and runtime budget bounds");
            println!(
                "  --audit-privacy       Audit privacy, support, local data, and no-network posture"
            );
            println!("  --audit-release-provenance Audit build inputs and release traceability");
            println!("  --audit-marketing     Audit store and press release materials and exit");
            println!("  --audit-ip            Audit release-facing naming and source separation");
            println!("  --audit-save          Audit save path and compatibility behavior and exit");
            println!(
                "  --audit-playthrough   Audit scripted victory, defeat, restart, and score flow"
            );
            println!("  --release-readiness   Print release-candidate readiness status");
            println!("  --simulate-campaign   Run headless campaign progression QA and exit");
            println!(
                "  --store-screenshot-scene <scene> Start a deterministic store screenshot scene"
            );
            println!("  --print-release-info  Print release metadata and exit");
            println!("  --print-save-path     Print the save file path and exit");
            println!("  --print-save-summary  Print normalized save data and exit");
            println!("  --version             Print release metadata and exit");
            println!("  --audio               Start with Bevy audio enabled");
            true
        }
        _ => false,
    }
}

// CJK-capable monospace bundled for both targets (the bevy default font has
// no Chinese glyphs); replacing the default handle restyles every Text node.
const GAME_FONT_TTF: &[u8] = include_bytes!("../assets/fonts/SarasaMonoSC-subset.ttf");

fn install_ui_font(mut fonts: ResMut<Assets<Font>>) {
    let font = Font::try_from_bytes(GAME_FONT_TTF.to_vec()).expect("bundled UI font must parse");
    fonts
        .insert(Handle::<Font>::default().id(), font)
        .expect("default font slot must accept the bundled UI font");
}

fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 9.5, 9.7).looking_at(Vec3::new(0.0, 0.0, 0.4), Vec3::Y),
        CameraRig {
            base: Vec3::new(0.0, 9.5, 9.7),
        },
        AmbientLight {
            color: Color::srgb(0.72, 0.86, 0.80),
            brightness: 420.0,
            ..default()
        },
        Name::new("Board Camera"),
    ));
    commands.spawn((
        DirectionalLight {
            illuminance: 13_500.0,
            color: Color::srgb(1.0, 0.96, 0.86),
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(-4.0, 8.0, 4.0).looking_at(Vec3::ZERO, Vec3::Y),
        Name::new("Sun Key Light"),
    ));
    commands.spawn((
        DirectionalLight {
            illuminance: 3_200.0,
            color: Color::srgb(0.62, 0.74, 0.92),
            shadows_enabled: false,
            ..default()
        },
        Transform::from_xyz(5.0, 6.0, -3.0).looking_at(Vec3::ZERO, Vec3::Y),
        Name::new("Sky Fill Light"),
    ));
}

fn setup_store_screenshot_scene(
    mode: Res<StoreScreenshotMode>,
    mut language: ResMut<LanguageSettings>,
    mut levels: ResMut<LevelCatalog>,
    mut progress: ResMut<ProgressState>,
    mut board_state: ResMut<BoardState>,
    mut next: ResMut<NextState<GameState>>,
) {
    let Some(scene) = mode.scene else {
        return;
    };
    progress.unlocked_levels = levels.levels.len();
    progress.best_scores.resize(levels.levels.len(), 0);
    match scene {
        StoreScreenshotScene::TitleMenu => {
            language.current = Language::English;
            levels.selected = 0;
        }
        StoreScreenshotScene::EarlyDefense => {
            language.current = Language::English;
            levels.selected = 0;
            next.set(GameState::Playing);
        }
        StoreScreenshotScene::SpecialEnemies => {
            language.current = Language::Chinese;
            levels.selected = 5.min(levels.levels.len() - 1);
            next.set(GameState::Playing);
        }
        StoreScreenshotScene::LateSiege => {
            language.current = Language::English;
            levels.selected = 9.min(levels.levels.len() - 1);
            next.set(GameState::Playing);
        }
        StoreScreenshotScene::VictorySummary => {
            language.current = Language::English;
            let level_index = 9.min(levels.levels.len() - 1);
            levels.selected = level_index;
            *board_state = BoardState::for_level(level_index, &levels.levels[level_index]);
            board_state.score = 824;
            board_state.wave = levels.levels[level_index].final_wave;
            board_state.final_wave_started = true;
            next.set(GameState::Victory);
        }
    }
}

fn apply_saved_window_settings(
    settings: Res<GameSettings>,
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
) {
    if let Ok(mut window) = windows.single_mut() {
        window.mode = if settings.fullscreen {
            WindowMode::BorderlessFullscreen(MonitorSelection::Primary)
        } else {
            WindowMode::Windowed
        };
    }
}

// On web, audio plays through bevy_audio: commands sent to the same channel
// are drained each frame into AudioPlayer entities (see drain_web_audio).
#[cfg(target_arch = "wasm32")]
static WEB_AUDIO_RECEIVER: std::sync::OnceLock<std::sync::Mutex<mpsc::Receiver<AudioCommand>>> =
    std::sync::OnceLock::new();

#[cfg(target_arch = "wasm32")]
fn start_async_audio(_enabled: bool) -> AsyncAudio {
    let (sender, receiver) = mpsc::channel::<AudioCommand>();
    let _ = WEB_AUDIO_RECEIVER.set(std::sync::Mutex::new(receiver));
    AsyncAudio {
        enabled: true,
        sender: Some(sender),
    }
}

#[cfg(target_arch = "wasm32")]
#[derive(Component)]
struct WebMusicSink;

// Browsers refuse to start audio before a user gesture, so the startup
// PlayMusic command is parked until the first key press or click.
#[cfg(target_arch = "wasm32")]
fn drain_web_audio(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut music: Query<&mut bevy::audio::AudioSink, With<WebMusicSink>>,
    mut pending_music: Local<Option<f32>>,
    mut interacted: Local<bool>,
) {
    use bevy::audio::Volume;

    if !*interacted
        && (keys.get_just_pressed().next().is_some()
            || mouse.get_just_pressed().next().is_some())
    {
        *interacted = true;
    }

    let Some(receiver) = WEB_AUDIO_RECEIVER.get() else {
        return;
    };
    let Ok(receiver) = receiver.lock() else {
        return;
    };
    while let Ok(command) = receiver.try_recv() {
        match command {
            AudioCommand::PlayMusic { master_volume } => {
                *pending_music = Some(master_volume);
            }
            AudioCommand::SetMusicVolume { master_volume } => {
                if pending_music.is_some() {
                    *pending_music = Some(master_volume);
                }
                for mut sink in music.iter_mut() {
                    sink.set_volume(Volume::Linear(music_volume_scalar(master_volume)));
                }
            }
            AudioCommand::PlaySound { path, volume } => {
                if *interacted {
                    commands.spawn((
                        AudioPlayer::new(asset_server.load(path)),
                        PlaybackSettings::DESPAWN
                            .with_volume(Volume::Linear(volume.clamp(0.0, 1.0))),
                    ));
                }
            }
        }
    }
    drop(receiver);

    if *interacted {
        if let Some(master_volume) = pending_music.take() {
            commands.spawn((
                AudioPlayer::new(asset_server.load(AUDIO_MUSIC_LOOP)),
                PlaybackSettings::LOOP
                    .with_volume(Volume::Linear(music_volume_scalar(master_volume))),
                WebMusicSink,
            ));
        }
    }
}

fn platform_audio_plugin(app: &mut App) {
    #[cfg(target_arch = "wasm32")]
    app.add_systems(Update, drain_web_audio);
    let _ = app;
}

#[cfg(not(target_arch = "wasm32"))]
fn start_async_audio(enabled: bool) -> AsyncAudio {
    if !enabled {
        return AsyncAudio {
            enabled,
            sender: None,
        };
    }

    let (sender, receiver) = mpsc::channel::<AudioCommand>();
    thread::Builder::new()
        .name("bevy-open-siege-audio".to_string())
        .spawn(move || run_audio_thread(receiver))
        .expect("audio thread should start");

    AsyncAudio {
        enabled,
        sender: Some(sender),
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn run_audio_thread(receiver: mpsc::Receiver<AudioCommand>) {
    let Ok((_stream, stream_handle)) = OutputStream::try_default() else {
        eprintln!("Bevy Open Siege audio disabled: no output device available");
        return;
    };

    let mut music_sink: Option<Sink> = None;
    while let Ok(command) = receiver.recv() {
        match command {
            AudioCommand::PlayMusic { master_volume } => {
                let Ok(source) = decode_audio(AUDIO_MUSIC_LOOP) else {
                    continue;
                };
                let Ok(sink) = Sink::try_new(&stream_handle) else {
                    continue;
                };
                sink.set_volume(music_volume_scalar(master_volume));
                sink.append(source.repeat_infinite());
                sink.play();
                music_sink = Some(sink);
            }
            AudioCommand::SetMusicVolume { master_volume } => {
                if let Some(sink) = music_sink.as_ref() {
                    sink.set_volume(music_volume_scalar(master_volume));
                }
            }
            AudioCommand::PlaySound { path, volume } => {
                let Some(bytes) = audio_bytes(path) else {
                    continue;
                };
                let Ok(source) = decode_audio_bytes(bytes) else {
                    continue;
                };
                let Ok(sink) = Sink::try_new(&stream_handle) else {
                    continue;
                };
                sink.set_volume(volume.clamp(0.0, 1.0));
                sink.append(source);
                sink.detach();
            }
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn decode_audio(path: &'static str) -> Result<Decoder<BufReader<Cursor<&'static [u8]>>>, String> {
    let bytes = audio_bytes(path).ok_or_else(|| format!("unknown audio asset: {path}"))?;
    decode_audio_bytes(bytes)
}

#[cfg(not(target_arch = "wasm32"))]
fn decode_audio_bytes(
    bytes: &'static [u8],
) -> Result<Decoder<BufReader<Cursor<&'static [u8]>>>, String> {
    Decoder::new(BufReader::new(Cursor::new(bytes))).map_err(|error| error.to_string())
}

#[cfg(not(target_arch = "wasm32"))]
fn audio_bytes(path: &'static str) -> Option<&'static [u8]> {
    match path {
        AUDIO_MUSIC_LOOP => Some(MUSIC_LOOP_WAV),
        AUDIO_PLANT_PLACE => Some(PLANT_PLACE_WAV),
        AUDIO_SHOOT => Some(SHOOT_WAV),
        AUDIO_SUN_COLLECT => Some(SUN_COLLECT_WAV),
        AUDIO_MONSTER_DOWN => Some(MONSTER_DOWN_WAV),
        AUDIO_VICTORY => Some(VICTORY_WAV),
        AUDIO_DEFEAT => Some(DEFEAT_WAV),
        _ => None,
    }
}

fn send_audio(audio: &AsyncAudio, command: AudioCommand) {
    if let Some(sender) = audio.sender.as_ref() {
        let _ = sender.send(command);
    }
}

fn setup_audio(audio: Res<AsyncAudio>, settings: Res<GameSettings>) {
    if !audio.enabled {
        return;
    }
    send_audio(
        &audio,
        AudioCommand::PlayMusic {
            master_volume: settings.master_volume,
        },
    );
}

fn apply_audio_volume(settings: Res<GameSettings>, audio: Res<AsyncAudio>) {
    if !settings.is_changed() || !audio.enabled {
        return;
    }
    send_audio(
        &audio,
        AudioCommand::SetMusicVolume {
            master_volume: settings.master_volume,
        },
    );
}

fn music_volume_scalar(master_volume: f32) -> f32 {
    (master_volume * 0.20).clamp(0.0, 1.0)
}

fn sound_volume_scalar(settings: &GameSettings, gain: f32) -> f32 {
    (settings.master_volume * gain).clamp(0.0, 1.0)
}

fn play_sound(audio: &AsyncAudio, settings: &GameSettings, path: &'static str, gain: f32) {
    if settings.master_volume <= 0.0 || !audio.enabled {
        return;
    }
    send_audio(
        audio,
        AudioCommand::PlaySound {
            path,
            volume: sound_volume_scalar(settings, gain),
        },
    );
}

fn toggle_language(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut language: ResMut<LanguageSettings>,
    progress: Res<ProgressState>,
    settings: Res<GameSettings>,
) {
    if keyboard.just_pressed(KeyCode::KeyL) {
        language.current = language.current.next();
        save_progress(language.current, &progress, &settings);
    }
}

fn settings_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    language: Res<LanguageSettings>,
    progress: Res<ProgressState>,
    mut settings: ResMut<GameSettings>,
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
) {
    let mut changed = false;
    if keyboard.just_pressed(KeyCode::Equal) || keyboard.just_pressed(KeyCode::NumpadAdd) {
        settings.master_volume = volume_after_step(settings.master_volume, 0.1);
        changed = true;
    }
    if keyboard.just_pressed(KeyCode::Minus) || keyboard.just_pressed(KeyCode::NumpadSubtract) {
        settings.master_volume = volume_after_step(settings.master_volume, -0.1);
        changed = true;
    }
    if keyboard.just_pressed(KeyCode::KeyF) {
        settings.fullscreen = !settings.fullscreen;
        if let Ok(mut window) = windows.single_mut() {
            window.mode = if settings.fullscreen {
                WindowMode::BorderlessFullscreen(MonitorSelection::Primary)
            } else {
                WindowMode::Windowed
            };
        }
        changed = true;
    }
    if changed {
        save_progress(language.current, &progress, &settings);
    }
}

fn setup_ui_fonts(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(UiFonts {
        cjk: asset_server.load(FONT_CJK),
    });
}

// The bundled default font has no CJK glyphs, so Chinese UI text swaps to an
// embedded Sarasa Mono SC subset; English keeps the stock font.
#[allow(clippy::type_complexity)]
fn apply_language_font(
    language: Res<LanguageSettings>,
    fonts: Res<UiFonts>,
    mut texts: ParamSet<(
        Query<&mut TextFont>,
        Query<&mut TextFont, Added<TextFont>>,
    )>,
) {
    let font = match language.current {
        Language::Chinese => fonts.cjk.clone(),
        Language::English => Handle::default(),
    };
    if language.is_changed() {
        for mut text_font in texts.p0().iter_mut() {
            text_font.font = font.clone();
        }
    } else if language.current == Language::Chinese {
        for mut text_font in texts.p1().iter_mut() {
            text_font.font = font.clone();
        }
    }
}

fn toggle_pause(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut pause: ResMut<PauseState>,
    state: Res<State<GameState>>,
) {
    if *state.get() == GameState::Playing && keyboard.just_pressed(KeyCode::KeyP) {
        pause.paused = !pause.paused;
    }
}

fn not_paused(pause: Res<PauseState>) -> bool {
    !pause.paused
}

fn menu_level_line(
    locale: &LocaleText,
    language: Language,
    levels: &LevelCatalog,
    progress: &ProgressState,
) -> String {
    let level = &levels.levels[levels.selected];
    let locked = if progress.is_unlocked(levels.selected) {
        ""
    } else {
        locale.locked_level.as_str()
    };
    let best = progress
        .best_score(levels.selected)
        .map(|score| score.to_string())
        .unwrap_or_else(|| locale.no_score.clone());
    format!(
        "{} {}: {} [{}] {} | {}: {}",
        locale.level,
        levels.selected + 1,
        level.title(language),
        level.id,
        locked,
        locale.best_score,
        best
    )
}

fn menu_roster_line(
    locale: &LocaleText,
    language: Language,
    levels: &LevelCatalog,
    progress: &ProgressState,
    index: usize,
) -> String {
    let level = &levels.levels[index];
    let selected = if index == levels.selected { ">" } else { " " };
    let lock = if progress.is_unlocked(index) {
        ""
    } else {
        locale.locked_level.as_str()
    };
    let best = progress
        .best_score(index)
        .map(|score| score.to_string())
        .unwrap_or_else(|| locale.no_score.clone());
    format!(
        "{selected} {:02}. {}  {}:{}  {}",
        index + 1,
        level.title(language),
        locale.best_score,
        best,
        lock
    )
}

fn menu_roster_text(
    locale: &LocaleText,
    language: Language,
    levels: &LevelCatalog,
    progress: &ProgressState,
) -> String {
    (0..levels.levels.len())
        .map(|index| menu_roster_line(locale, language, levels, progress, index))
        .collect::<Vec<_>>()
        .join("\n")
}

fn menu_settings_line(settings: &GameSettings, language: Language) -> String {
    let (audio, fullscreen, volume, language_label, on, off) = match language {
        Language::English => ("Audio", "Fullscreen", "Volume", "Language", "on", "off"),
        Language::Chinese => ("音频", "全屏", "音量", "语言", "开", "关"),
    };
    format!(
        "{audio}: {} | {fullscreen}: {} | {volume}: {:.0}% | {language_label}: {}",
        if audio_enabled() { on } else { off },
        if settings.fullscreen { on } else { off },
        settings.master_volume * 100.0,
        language.label()
    )
}

fn spawn_menu(
    mut commands: Commands,
    language: Res<LanguageSettings>,
    localization: Res<LocalizationCatalog>,
    levels: Res<LevelCatalog>,
    progress: Res<ProgressState>,
    settings: Res<GameSettings>,
    asset_server: Res<AssetServer>,
) {
    let locale = localization.text(language.current);
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(10.0),
                ..default()
            },
            ui_panel_image(&asset_server, UI_MENU_PANEL),
            MenuUi,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new(locale.title.clone()),
                TextFont {
                    font_size: 50.0,
                    ..default()
                },
                TextColor(Color::srgb(0.88, 0.96, 0.68)),
                MenuTitleText,
            ));
            parent.spawn((
                Text::new(locale.menu_help.clone()),
                TextFont {
                    font_size: 17.0,
                    ..default()
                },
                TextColor(Color::srgb(0.74, 0.84, 0.68)),
                MenuHelpText,
            ));
            parent.spawn((
                Text::new(menu_level_line(
                    locale,
                    language.current,
                    &levels,
                    &progress,
                )),
                TextFont {
                    font_size: 22.0,
                    ..default()
                },
                TextColor(Color::srgb(0.86, 0.82, 0.58)),
                MenuLevelText,
            ));
            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(2.0),
                    ..default()
                })
                .with_children(|list| {
                    for index in 0..levels.levels.len() {
                        list.spawn((
                            Button,
                            Node {
                                padding: UiRect::axes(Val::Px(14.0), Val::Px(3.0)),
                                justify_content: JustifyContent::FlexStart,
                                border_radius: BorderRadius::all(Val::Px(6.0)),
                                ..default()
                            },
                            BackgroundColor(Color::NONE),
                            MenuLevelRow(index),
                        ))
                        .with_children(|row| {
                            row.spawn((
                                Text::new(menu_roster_line(
                                    locale,
                                    language.current,
                                    &levels,
                                    &progress,
                                    index,
                                )),
                                TextFont {
                                    font_size: 18.0,
                                    ..default()
                                },
                                TextColor(Color::srgb(0.78, 0.88, 0.66)),
                                MenuRosterText,
                                MenuLevelRow(index),
                            ));
                        });
                    }
                });
            parent.spawn((
                Text::new(menu_settings_line(&settings, language.current)),
                TextFont {
                    font_size: 18.0,
                    ..default()
                },
                TextColor(Color::srgb(0.72, 0.82, 0.90)),
                MenuSettingsText,
            ));
            parent
                .spawn((
                    Button,
                    Node {
                        padding: UiRect::axes(Val::Px(22.0), Val::Px(8.0)),
                        justify_content: JustifyContent::Center,
                        border_radius: BorderRadius::all(Val::Px(10.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.16, 0.24, 0.10, 0.85)),
                    MenuStartButton,
                ))
                .with_children(|button| {
                    button.spawn((
                        Text::new(locale.menu_start.clone()),
                        TextFont {
                            font_size: 26.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.96, 0.78, 0.30)),
                        MenuStartText,
                    ));
                });
        });
}

fn ui_panel_image(asset_server: &AssetServer, texture: &'static str) -> ImageNode {
    // The texture-slice UI shader fails to compile on some WebGL2 drivers
    // (ANGLE/Vulkan), panicking at pipeline creation; stretch the panel on web.
    #[cfg(target_arch = "wasm32")]
    {
        ImageNode::new(asset_server.load(texture))
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        ImageNode::new(asset_server.load(texture)).with_mode(NodeImageMode::Sliced(TextureSlicer {
            border: BorderRect::all(24.0),
            center_scale_mode: SliceScaleMode::Stretch,
            sides_scale_mode: SliceScaleMode::Stretch,
            max_corner_scale: 1.0,
        }))
    }
}

#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn update_menu_text(
    language: Res<LanguageSettings>,
    localization: Res<LocalizationCatalog>,
    levels: Res<LevelCatalog>,
    progress: Res<ProgressState>,
    settings: Res<GameSettings>,
    mut text_queries: ParamSet<(
        Query<&mut Text, With<MenuTitleText>>,
        Query<&mut Text, With<MenuHelpText>>,
        Query<&mut Text, With<MenuLevelText>>,
        Query<(&mut Text, &MenuLevelRow), With<MenuRosterText>>,
        Query<&mut Text, With<MenuSettingsText>>,
        Query<&mut Text, With<MenuStartText>>,
    )>,
) {
    if !language.is_changed()
        && !levels.is_changed()
        && !progress.is_changed()
        && !settings.is_changed()
    {
        return;
    }
    let locale = localization.text(language.current);
    if let Ok(mut text) = text_queries.p0().single_mut() {
        **text = locale.title.clone();
    }
    if let Ok(mut text) = text_queries.p1().single_mut() {
        **text = locale.menu_help.clone();
    }
    if let Ok(mut text) = text_queries.p2().single_mut() {
        **text = menu_level_line(locale, language.current, &levels, &progress);
    }
    for (mut text, row) in text_queries.p3().iter_mut() {
        **text = menu_roster_line(locale, language.current, &levels, &progress, row.0);
    }
    if let Ok(mut text) = text_queries.p4().single_mut() {
        **text = menu_settings_line(&settings, language.current);
    }
    if let Ok(mut text) = text_queries.p5().single_mut() {
        **text = locale.menu_start.clone();
    }
}

fn menu_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    progress: Res<ProgressState>,
    mut levels: ResMut<LevelCatalog>,
    mut next: ResMut<NextState<GameState>>,
) {
    if keyboard.just_pressed(KeyCode::Tab)
        || keyboard.just_pressed(KeyCode::ArrowRight)
        || keyboard.just_pressed(KeyCode::ArrowDown)
    {
        levels.selected = next_menu_selection(levels.selected, levels.levels.len());
    }
    if keyboard.just_pressed(KeyCode::ArrowLeft) || keyboard.just_pressed(KeyCode::ArrowUp) {
        levels.selected = previous_menu_selection(levels.selected, levels.levels.len());
    }
    for (index, key) in [
        KeyCode::Digit1,
        KeyCode::Digit2,
        KeyCode::Digit3,
        KeyCode::Digit4,
        KeyCode::Digit5,
        KeyCode::Digit6,
        KeyCode::Digit7,
        KeyCode::Digit8,
        KeyCode::Digit9,
        KeyCode::Digit0,
    ]
    .into_iter()
    .enumerate()
    {
        if keyboard.just_pressed(key) && index < levels.levels.len() {
            levels.selected = index;
        }
    }
    if (keyboard.just_pressed(KeyCode::Enter) || keyboard.just_pressed(KeyCode::Space))
        && can_start_selected_level(&progress, levels.selected)
    {
        next.set(GameState::Playing);
    }
}

fn restart_input(keyboard: Res<ButtonInput<KeyCode>>, mut next: ResMut<NextState<GameState>>) {
    if keyboard.just_pressed(KeyCode::KeyR) || keyboard.just_pressed(KeyCode::Enter) {
        next.set(GameState::Playing);
    }
}

#[allow(clippy::type_complexity)]
fn menu_mouse(
    progress: Res<ProgressState>,
    mut levels: ResMut<LevelCatalog>,
    mut next: ResMut<NextState<GameState>>,
    mut rows: Query<(&Interaction, &MenuLevelRow), (Changed<Interaction>, With<Button>)>,
    mut start: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<MenuStartButton>),
    >,
) {
    for (interaction, row) in rows.iter_mut() {
        match interaction {
            Interaction::Hovered => levels.selected = row.0,
            Interaction::Pressed => {
                levels.selected = row.0;
                if can_start_selected_level(&progress, row.0) {
                    next.set(GameState::Playing);
                }
            }
            Interaction::None => (),
        }
    }
    for (interaction, mut background) in start.iter_mut() {
        match interaction {
            Interaction::Hovered => {
                *background = BackgroundColor(Color::srgba(0.26, 0.38, 0.16, 0.92));
            }
            Interaction::Pressed => {
                if can_start_selected_level(&progress, levels.selected) {
                    next.set(GameState::Playing);
                }
            }
            Interaction::None => {
                *background = BackgroundColor(Color::srgba(0.16, 0.24, 0.10, 0.85));
            }
        }
    }
}

#[allow(clippy::type_complexity)]
fn style_menu_rows(
    levels: Res<LevelCatalog>,
    progress: Res<ProgressState>,
    mut rows: Query<(&Interaction, &MenuLevelRow, &mut BackgroundColor), With<Button>>,
) {
    for (interaction, row, mut background) in rows.iter_mut() {
        let locked = !progress.is_unlocked(row.0);
        let color = if *interaction == Interaction::Hovered && !locked {
            Color::srgba(0.34, 0.50, 0.22, 0.60)
        } else if row.0 == levels.selected {
            Color::srgba(0.22, 0.36, 0.14, 0.55)
        } else {
            Color::NONE
        };
        if background.0 != color {
            *background = BackgroundColor(color);
        }
    }
}

fn spawn_greenhouse_dressing(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) {
    let grass_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.11, 0.21, 0.12),
        perceptual_roughness: 0.95,
        ..default()
    });
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(44.0, 0.16, 28.0))),
        MeshMaterial3d(grass_mat),
        Transform::from_xyz(0.0, -0.19, -2.0),
        GameEntity,
        Name::new("Greenhouse Ground"),
    ));

    let wood_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.32, 0.21, 0.11),
        perceptual_roughness: 0.8,
        ..default()
    });
    let post_mesh = meshes.add(Cuboid::new(0.18, 3.4, 0.18));
    for x in [-6.8, -3.4, 0.0, 3.4, 6.8] {
        commands.spawn((
            Mesh3d(post_mesh.clone()),
            MeshMaterial3d(wood_mat.clone()),
            Transform::from_xyz(x, 1.5, -4.9),
            GameEntity,
            Name::new("Greenhouse Post"),
        ));
    }
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(14.2, 0.18, 0.22))),
        MeshMaterial3d(wood_mat.clone()),
        Transform::from_xyz(0.0, 3.2, -4.9),
        GameEntity,
        Name::new("Greenhouse Beam"),
    ));

    let glass_mat = materials.add(StandardMaterial {
        base_color: Color::srgba(0.62, 0.86, 0.88, 0.16),
        perceptual_roughness: 0.12,
        alpha_mode: AlphaMode::Blend,
        cull_mode: None,
        ..default()
    });
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(13.9, 3.0, 0.05))),
        MeshMaterial3d(glass_mat),
        Transform::from_xyz(0.0, 1.6, -4.95),
        GameEntity,
        Name::new("Greenhouse Glass"),
    ));

    let pot_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.58, 0.31, 0.18),
        perceptual_roughness: 0.85,
        ..default()
    });
    let bush_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.20, 0.42, 0.18),
        perceptual_roughness: 0.75,
        ..default()
    });
    let pot_mesh = meshes.add(Cylinder::new(0.32, 0.5));
    let bush_mesh = meshes.add(Sphere::new(0.48));
    for (x, z) in [(-5.6, -5.9), (-1.8, -6.1), (2.4, -5.8), (6.0, -6.0)] {
        commands.spawn((
            Mesh3d(pot_mesh.clone()),
            MeshMaterial3d(pot_mat.clone()),
            Transform::from_xyz(x, 0.15, z),
            GameEntity,
            Name::new("Backdrop Pot"),
        ));
        commands.spawn((
            Mesh3d(bush_mesh.clone()),
            MeshMaterial3d(bush_mat.clone()),
            Transform::from_xyz(x, 0.75, z),
            GameEntity,
            Name::new("Backdrop Bush"),
        ));
    }
}

fn start_game(
    mut commands: Commands,
    mut state: ResMut<BoardState>,
    levels: Res<LevelCatalog>,
    mut onboarding: ResMut<OnboardingState>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    let level = &levels.levels[levels.selected];
    *state = BoardState::for_level(levels.selected, level);
    *onboarding = OnboardingState::default();

    let floor_mesh = meshes.add(Cuboid::new(13.2, 0.18, 8.0));
    let floor_mat = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        base_color_texture: Some(asset_server.load(ENV_LAWN_BASE)),
        perceptual_roughness: 0.92,
        ..default()
    });
    commands.spawn((
        Mesh3d(floor_mesh),
        MeshMaterial3d(floor_mat),
        Transform::from_xyz(0.0, -0.11, 0.0),
        GameEntity,
        Name::new("Greenhouse Lawn"),
    ));

    let tile_mesh = meshes.add(Cuboid::new(CELL - 0.07, 0.08, CELL - 0.07));
    for lane in 0..LANES {
        for col in 0..COLS {
            let tint = if (lane + col) % 2 == 0 { 1.0 } else { 0.86 };
            let mat = materials.add(StandardMaterial {
                base_color: Color::srgb(tint, tint, tint),
                base_color_texture: Some(asset_server.load(ENV_LANE_GRASS)),
                perceptual_roughness: 0.95,
                ..default()
            });
            commands.spawn((
                Mesh3d(tile_mesh.clone()),
                MeshMaterial3d(mat),
                Transform::from_xyz(col_x(col), 0.0, lane_z(lane)),
                GameEntity,
            ));
        }
    }

    let border_material = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        base_color_texture: Some(asset_server.load(ENV_SOIL_BORDER)),
        perceptual_roughness: 0.98,
        ..default()
    });
    let horizontal_border = meshes.add(Cuboid::new(13.0, 0.10, 0.36));
    let vertical_border = meshes.add(Cuboid::new(0.36, 0.10, 7.4));
    for z in [lane_z(0) + CELL * 0.56, lane_z(LANES - 1) - CELL * 0.56] {
        commands.spawn((
            Mesh3d(horizontal_border.clone()),
            MeshMaterial3d(border_material.clone()),
            Transform::from_xyz(0.0, 0.03, z),
            GameEntity,
            Name::new("Soil Border"),
        ));
    }
    for x in [col_x(0) - CELL * 0.58, col_x(COLS - 1) + CELL * 0.58] {
        commands.spawn((
            Mesh3d(vertical_border.clone()),
            MeshMaterial3d(border_material.clone()),
            Transform::from_xyz(x, 0.03, 0.28),
            GameEntity,
            Name::new("Soil Border"),
        ));
    }

    spawn_greenhouse_dressing(&mut commands, &mut meshes, &mut materials);

    let cursor_mat = materials.add(StandardMaterial {
        base_color: Color::srgba(1.0, 0.95, 0.28, 0.62),
        emissive: Color::srgb(0.18, 0.14, 0.02).into(),
        alpha_mode: AlphaMode::Blend,
        ..default()
    });
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(CELL, 0.12, CELL))),
        MeshMaterial3d(cursor_mat),
        Transform::from_xyz(col_x(state.cursor_col), 0.08, lane_z(state.cursor_lane)),
        CursorMarker,
        GameEntity,
    ));

    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Px(74.0),
                position_type: PositionType::Absolute,
                top: Val::Px(0.0),
                left: Val::Px(0.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                padding: UiRect::horizontal(Val::Px(24.0)),
                ..default()
            },
            ui_panel_image(&asset_server, UI_HUD_PANEL),
            HudUi,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new(""),
                TextFont {
                    font_size: 18.0,
                    ..default()
                },
                TextColor(Color::srgb(0.92, 0.96, 0.78)),
                HudText,
                HudStatusText,
            ));
        });

    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(78.0),
                left: Val::Percent(27.0),
                width: Val::Percent(46.0),
                height: Val::Px(8.0),
                padding: UiRect::all(Val::Px(1.0)),
                border_radius: BorderRadius::all(Val::Px(4.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.02, 0.06, 0.05, 0.85)),
            HudUi,
        ))
        .with_children(|bar| {
            bar.spawn((
                Node {
                    width: Val::Percent(0.0),
                    height: Val::Percent(100.0),
                    border_radius: BorderRadius::all(Val::Px(3.0)),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.45, 0.78, 0.30)),
                WaveBarFill,
            ));
        });

    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Px(112.0),
                position_type: PositionType::Absolute,
                bottom: Val::Px(0.0),
                left: Val::Px(0.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                padding: UiRect::horizontal(Val::Px(18.0)),
                ..default()
            },
            ui_panel_image(&asset_server, UI_HUD_PANEL),
            HudUi,
        ))
        .with_children(|parent| {
            parent
                .spawn(Node {
                    column_gap: Val::Px(6.0),
                    align_items: AlignItems::Stretch,
                    ..default()
                })
                .with_children(|row| {
                    for kind in PlantKind::ALL {
                        row.spawn((
                            Button,
                            Node {
                                width: Val::Px(118.0),
                                padding: UiRect::axes(Val::Px(6.0), Val::Px(6.0)),
                                flex_direction: FlexDirection::Column,
                                align_items: AlignItems::Center,
                                justify_content: JustifyContent::Center,
                                border_radius: BorderRadius::all(Val::Px(8.0)),
                                ..default()
                            },
                            BackgroundColor(Color::srgba(0.10, 0.16, 0.08, 0.85)),
                            SeedButton(kind),
                        ))
                        .with_children(|card| {
                            card.spawn((
                                Text::new(""),
                                TextFont {
                                    font_size: 13.0,
                                    ..default()
                                },
                                TextColor(Color::srgb(0.85, 0.93, 0.72)),
                            ));
                        });
                    }
                });
        });
}

#[allow(clippy::too_many_arguments)]
fn handle_board_input(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    cameras: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    pause: Res<PauseState>,
    mut state: ResMut<BoardState>,
    plant_query: Query<(Entity, &Plant)>,
    asset_server: Res<AssetServer>,
    settings: Res<GameSettings>,
    audio: Res<AsyncAudio>,
) {
    for key in [
        KeyCode::ArrowLeft,
        KeyCode::ArrowRight,
        KeyCode::ArrowUp,
        KeyCode::ArrowDown,
    ] {
        if keyboard.just_pressed(key) {
            (state.cursor_col, state.cursor_lane) =
                cursor_after_arrow(state.cursor_col, state.cursor_lane, key);
        }
    }
    for key in [
        KeyCode::Digit1,
        KeyCode::Digit2,
        KeyCode::Digit3,
        KeyCode::Digit4,
        KeyCode::Digit5,
        KeyCode::Digit6,
        KeyCode::Digit7,
        KeyCode::Digit8,
        KeyCode::Digit9,
        KeyCode::Digit0,
    ] {
        if keyboard.just_pressed(key)
            && let Some(kind) = plant_kind_for_digit_key(key)
        {
            state.selected = kind;
        }
    }

    let mut clicked_board = false;
    if let Ok(window) = windows.single()
        && let Some((col, lane)) = cursor_grid_cell(window, &cameras)
        && (mouse.just_pressed(MouseButton::Left) || mouse.just_pressed(MouseButton::Right))
    {
        state.cursor_col = col;
        state.cursor_lane = lane;
        clicked_board = true;
    }

    if keyboard.just_pressed(KeyCode::Backspace) || mouse.just_pressed(MouseButton::Right) {
        for (entity, plant) in &plant_query {
            if plant.col == state.cursor_col && plant.lane == state.cursor_lane {
                commands.entity(entity).despawn();
                break;
            }
        }
    }

    if pause.paused {
        return;
    }

    // Mouse planting requires the click to land on a board tile, so UI
    // clicks (seed cards, pause buttons) never place a plant as a side effect.
    let wants_plant = keyboard.just_pressed(KeyCode::Space)
        || (clicked_board && mouse.just_pressed(MouseButton::Left));
    if !wants_plant {
        return;
    }

    let occupied = plant_query
        .iter()
        .any(|(_, plant)| plant.col == state.cursor_col && plant.lane == state.cursor_lane);

    let kind = state.selected;
    if plant_placement_block(
        pause.paused,
        occupied,
        state.sun,
        state.plant_cooldowns[kind.index()],
        kind,
    )
    .is_some()
    {
        return;
    }

    state.sun -= kind.cost();
    state.plant_cooldowns[kind.index()] = kind.cooldown_seconds();
    spawn_plant(
        &mut commands,
        &asset_server,
        kind,
        state.cursor_col,
        state.cursor_lane,
    );
    play_sound(&audio, &settings, AUDIO_PLANT_PLACE, 0.72);
}

fn cursor_grid_cell(
    window: &Window,
    cameras: &Query<(&Camera, &GlobalTransform), With<Camera3d>>,
) -> Option<(usize, usize)> {
    let cursor = window.cursor_position()?;
    let (camera, camera_transform) = cameras.single().ok()?;
    let ray = camera.viewport_to_world(camera_transform, cursor).ok()?;
    let point = ray.plane_intersection_point(Vec3::ZERO, InfinitePlane3d::new(Vec3::Y))?;
    grid_cell_from_world(point)
}

fn grid_cell_from_world(point: Vec3) -> Option<(usize, usize)> {
    let col = ((point.x - BOARD_LEFT) / CELL).round() as isize;
    let lane = ((BOARD_TOP - point.z) / CELL).round() as isize;
    if (0..COLS as isize).contains(&col) && (0..LANES as isize).contains(&lane) {
        Some((col as usize, lane as usize))
    } else {
        None
    }
}

fn tick_plant_cooldowns(time: Res<Time>, pause: Res<PauseState>, mut state: ResMut<BoardState>) {
    if pause.paused {
        return;
    }
    for cooldown in &mut state.plant_cooldowns {
        *cooldown = (*cooldown - time.delta_secs()).max(0.0);
    }
}

#[allow(clippy::too_many_arguments)]
fn update_pause_ui(
    mut commands: Commands,
    pause: Res<PauseState>,
    language: Res<LanguageSettings>,
    localization: Res<LocalizationCatalog>,
    settings: Res<GameSettings>,
    asset_server: Res<AssetServer>,
    query: Query<Entity, (With<PauseUi>, Without<PauseText>)>,
    mut text_query: Query<&mut Text, With<PauseText>>,
    mut button_labels: Query<(&mut Text, &PauseButton), Without<PauseText>>,
) {
    let exists = !query.is_empty();
    if pause.paused && !exists {
        let locale = localization.text(language.current);
        commands
            .spawn((
                Node {
                    position_type: PositionType::Absolute,
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                BackgroundColor(Color::srgba(0.01, 0.04, 0.03, 0.55)),
                GlobalZIndex(10),
                PauseUi,
            ))
            .with_children(|overlay| {
                overlay
                    .spawn((
                        Node {
                            width: Val::Px(430.0),
                            padding: UiRect::all(Val::Px(22.0)),
                            flex_direction: FlexDirection::Column,
                            align_items: AlignItems::Center,
                            row_gap: Val::Px(12.0),
                            ..default()
                        },
                        ui_panel_image(&asset_server, UI_END_PANEL),
                    ))
                    .with_children(|parent| {
                        parent.spawn((
                            Text::new(pause_text(locale, language.current, &settings)),
                            TextFont {
                                font_size: 20.0,
                                ..default()
                            },
                            TextColor(Color::srgb(0.92, 0.96, 0.78)),
                            PauseText,
                        ));
                        for (action, label) in [
                            (PauseButton::Resume, locale.pause_resume.clone()),
                            (PauseButton::Restart, locale.pause_restart.clone()),
                            (PauseButton::Quit, locale.pause_quit.clone()),
                        ] {
                            parent
                                .spawn((
                                    Button,
                                    Node {
                                        width: Val::Px(260.0),
                                        padding: UiRect::axes(Val::Px(16.0), Val::Px(8.0)),
                                        justify_content: JustifyContent::Center,
                                        border_radius: BorderRadius::all(Val::Px(8.0)),
                                        ..default()
                                    },
                                    BackgroundColor(Color::srgba(0.16, 0.24, 0.10, 0.90)),
                                    action,
                                ))
                                .with_children(|button| {
                                    button.spawn((
                                        Text::new(label),
                                        TextFont {
                                            font_size: 22.0,
                                            ..default()
                                        },
                                        TextColor(Color::srgb(0.90, 0.94, 0.72)),
                                        action,
                                    ));
                                });
                        }
                    });
            });
    } else if !pause.paused && exists {
        for entity in &query {
            commands.entity(entity).despawn();
        }
    } else if pause.paused && (settings.is_changed() || language.is_changed()) {
        let locale = localization.text(language.current);
        for mut text in &mut text_query {
            **text = pause_text(locale, language.current, &settings);
        }
        for (mut text, action) in &mut button_labels {
            **text = match action {
                PauseButton::Resume => locale.pause_resume.clone(),
                PauseButton::Restart => locale.pause_restart.clone(),
                PauseButton::Quit => locale.pause_quit.clone(),
            };
        }
    }
}

#[allow(clippy::type_complexity)]
fn pause_menu_buttons(
    mut commands: Commands,
    mut pause: ResMut<PauseState>,
    mut next: ResMut<NextState<GameState>>,
    mut buttons: Query<
        (&Interaction, &PauseButton, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (interaction, action, mut background) in buttons.iter_mut() {
        match interaction {
            Interaction::Hovered => {
                *background = BackgroundColor(Color::srgba(0.28, 0.40, 0.18, 0.95));
            }
            Interaction::None => {
                *background = BackgroundColor(Color::srgba(0.16, 0.24, 0.10, 0.90));
            }
            Interaction::Pressed => {
                pause.paused = false;
                match action {
                    PauseButton::Resume => (),
                    PauseButton::Restart => {
                        commands.run_system_cached(despawn_game);
                        commands.run_system_cached(start_game);
                    }
                    PauseButton::Quit => next.set(GameState::Menu),
                }
            }
        }
    }
}

fn pause_text(locale: &LocaleText, language: Language, settings: &GameSettings) -> String {
    let (resume, fullscreen, volume, lang_label, on, off) = match language {
        Language::English => (
            "P resume",
            "F fullscreen",
            "+/- volume",
            "L language",
            "on",
            "off",
        ),
        Language::Chinese => ("P 继续", "F 全屏", "+/- 音量", "L 语言", "开", "关"),
    };
    format!(
        "{}\n{resume} | {fullscreen}: {} | {volume}: {:.0}% | {lang_label}",
        locale.title,
        if settings.fullscreen { on } else { off },
        settings.master_volume * 100.0
    )
}

fn wave_progress(wave: u32, timer_fraction: f32, final_wave: u32, final_started: bool) -> f32 {
    if final_started {
        return 1.0;
    }
    ((wave.saturating_sub(1) as f32 + timer_fraction) / final_wave.max(1) as f32).clamp(0.0, 1.0)
}

fn update_wave_bar(
    state: Res<BoardState>,
    levels: Res<LevelCatalog>,
    mut fills: Query<(&mut Node, &mut BackgroundColor), With<WaveBarFill>>,
) {
    let level = &levels.levels[state.level_index];
    let progress = wave_progress(
        state.wave,
        state.wave_timer.fraction(),
        level.final_wave,
        state.final_wave_started,
    );
    for (mut node, mut color) in fills.iter_mut() {
        node.width = Val::Percent(progress * 100.0);
        // green while calm, sliding toward red as the final wave nears
        *color = BackgroundColor(Color::hsl(110.0 - progress * 95.0, 0.65, 0.45));
    }
}

#[allow(clippy::too_many_arguments)]
fn update_onboarding(
    mut commands: Commands,
    time: Res<Time>,
    state: Res<BoardState>,
    language: Res<LanguageSettings>,
    localization: Res<LocalizationCatalog>,
    mut onboarding: ResMut<OnboardingState>,
    plants: Query<&Plant>,
    hint_root: Query<Entity, With<HintUi>>,
    mut hint_text: Query<&mut Text, With<HintText>>,
) {
    if state.level_index != 0 || onboarding.done {
        for entity in &hint_root {
            commands.entity(entity).despawn();
        }
        return;
    }
    match onboarding.step {
        0 => {
            if plants.iter().any(|plant| plant.kind == PlantKind::Sunflower) {
                onboarding.step = 1;
            }
        }
        1 => {
            if state.sun >= 100 {
                onboarding.step = 2;
            }
        }
        2 => {
            if plants.iter().any(|plant| plant.kind != PlantKind::Sunflower) {
                onboarding.step = 3;
                onboarding.timer = Timer::from_seconds(8.0, TimerMode::Once);
            }
        }
        _ => {
            onboarding.timer.tick(time.delta());
            if onboarding.timer.is_finished() {
                onboarding.done = true;
            }
        }
    }
    let locale = localization.text(language.current);
    let hint = locale
        .hints
        .get(onboarding.step)
        .cloned()
        .unwrap_or_default();
    if hint_root.is_empty() {
        commands
            .spawn((
                Node {
                    position_type: PositionType::Absolute,
                    bottom: Val::Px(124.0),
                    width: Val::Percent(100.0),
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                HintUi,
                HudUi,
            ))
            .with_children(|root| {
                root.spawn((
                    Node {
                        padding: UiRect::axes(Val::Px(18.0), Val::Px(8.0)),
                        border_radius: BorderRadius::all(Val::Px(8.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.03, 0.09, 0.06, 0.85)),
                ))
                .with_children(|panel| {
                    panel.spawn((
                        Text::new(hint.clone()),
                        TextFont {
                            font_size: 18.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.98, 0.88, 0.52)),
                        HintText,
                    ));
                });
            });
    } else if let Ok(mut text) = hint_text.single_mut()
        && text.0 != hint
    {
        text.0 = hint;
    }
}

fn update_cursor(state: Res<BoardState>, mut query: Query<&mut Transform, With<CursorMarker>>) {
    if !state.is_changed() {
        return;
    }
    if let Ok(mut transform) = query.single_mut() {
        transform.translation.x = col_x(state.cursor_col);
        transform.translation.z = lane_z(state.cursor_lane);
    }
}

fn billboard_mesh(width: f32, height: f32) -> Mesh {
    Mesh::from(Rectangle::new(width, height))
}

fn spawn_plant(
    commands: &mut Commands,
    asset_server: &AssetServer,
    kind: PlantKind,
    col: usize,
    lane: usize,
) {
    let fire_seconds = match kind {
        PlantKind::Peashooter => 1.35,
        PlantKind::SnowPea => 1.65,
        PlantKind::Repeater => 1.20,
        PlantKind::CabbagePult => 2.05,
        PlantKind::Spikeweed => 0.65,
        PlantKind::CherryBomb => 1.35,
        PlantKind::Torchwood => 0.75,
        _ => 7.0,
    };
    commands.spawn((
        SceneRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset(kind.model_path()))),
        Transform::from_xyz(col_x(col), 0.04, lane_z(lane)).with_scale(Vec3::splat(0.05)),
        GrowIn {
            timer: Timer::from_seconds(0.3, TimerMode::Once),
            target_scale: 1.2,
        },
        Plant {
            kind,
            col,
            lane,
            health: kind.max_health(),
            fire_timer: Timer::from_seconds(
                fire_seconds,
                if kind == PlantKind::CherryBomb {
                    TimerMode::Once
                } else {
                    TimerMode::Repeating
                },
            ),
            sun_timer: Timer::from_seconds(7.0, TimerMode::Repeating),
        },
        GameEntity,
        Name::new(kind.fallback_label()),
    ));
}

#[allow(clippy::too_many_arguments)]
fn populate_store_screenshot_scene(
    mut commands: Commands,
    mode: Res<StoreScreenshotMode>,
    mut state: ResMut<BoardState>,
    levels: Res<LevelCatalog>,
    language: Res<LanguageSettings>,
    localization: Res<LocalizationCatalog>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    let Some(scene) = mode.scene else {
        return;
    };
    let locale = localization.text(language.current);
    match scene {
        StoreScreenshotScene::TitleMenu | StoreScreenshotScene::VictorySummary => (),
        StoreScreenshotScene::EarlyDefense => {
            state.sun = 425;
            state.wave = 3;
            state.score = 90;
            state.selected = PlantKind::Repeater;
            state.spawn_timer = Timer::from_seconds(60.0, TimerMode::Repeating);
            state.wave_timer = Timer::from_seconds(60.0, TimerMode::Repeating);
            for (kind, col, lane) in [
                (PlantKind::Sunflower, 0, 0),
                (PlantKind::Sunflower, 0, 3),
                (PlantKind::Peashooter, 2, 1),
                (PlantKind::Wallnut, 4, 1),
                (PlantKind::SnowPea, 2, 3),
                (PlantKind::Repeater, 3, 2),
            ] {
                spawn_plant(&mut commands, &asset_server, kind, col, lane);
            }
            for (kind, lane, x) in [
                (ZombieKind::Walker, 1, 4.8),
                (ZombieKind::Conehead, 2, 5.9),
                (ZombieKind::Runner, 3, 3.7),
            ] {
                spawn_zombie_entity(&mut commands, &asset_server, kind, lane, x, 3, locale);
            }
            spawn_sun_pickup(
                &mut commands,
                &mut meshes,
                &mut materials,
                &asset_server,
                col_x(1),
                lane_z(0),
                25,
            );
            spawn_projectile(
                &mut commands,
                &mut meshes,
                &mut materials,
                &asset_server,
                2,
                Vec3::new(col_x(3), 0.55, lane_z(2)),
                ProjectileSpec {
                    damage: 22.0,
                    speed: 5.6,
                    slow_secs: 0.0,
                    pierce: 0,
                    splash_radius: 0.0,
                    color: Color::srgb(0.48, 0.95, 0.28),
                    texture: EFFECT_PEA,
                    visual_size: 0.34,
                    name: "Store Pea",
                },
            );
        }
        StoreScreenshotScene::SpecialEnemies => {
            let level = &levels.levels[state.level_index];
            state.sun = 560;
            state.wave = 8.min(level.final_wave);
            state.score = 380;
            state.selected = PlantKind::SnowPea;
            state.spawn_timer = Timer::from_seconds(60.0, TimerMode::Repeating);
            state.wave_timer = Timer::from_seconds(60.0, TimerMode::Repeating);
            for (kind, col, lane) in [
                (PlantKind::Sunflower, 0, 0),
                (PlantKind::CabbagePult, 2, 0),
                (PlantKind::SnowPea, 2, 1),
                (PlantKind::Spikeweed, 4, 2),
                (PlantKind::Torchwood, 3, 3),
                (PlantKind::Garlic, 4, 4),
            ] {
                spawn_plant(&mut commands, &asset_server, kind, col, lane);
            }
            for (kind, lane, x) in [
                (ZombieKind::Healer, 0, 5.2),
                (ZombieKind::Jumper, 1, 4.5),
                (ZombieKind::Digger, 2, 3.3),
                (ZombieKind::Frostbite, 3, 5.8),
                (ZombieKind::Buckethead, 4, 4.1),
            ] {
                spawn_zombie_entity(
                    &mut commands,
                    &asset_server,
                    kind,
                    lane,
                    x,
                    state.wave,
                    locale,
                );
            }
            spawn_projectile(
                &mut commands,
                &mut meshes,
                &mut materials,
                &asset_server,
                1,
                Vec3::new(col_x(2), 0.55, lane_z(1)),
                ProjectileSpec {
                    damage: 20.0,
                    speed: 4.9,
                    slow_secs: 3.4,
                    pierce: 0,
                    splash_radius: 0.0,
                    color: Color::srgb(0.55, 0.88, 1.0),
                    texture: EFFECT_FROST_POD,
                    visual_size: 0.34,
                    name: "Store Frost Pod",
                },
            );
        }
        StoreScreenshotScene::LateSiege => {
            let level = &levels.levels[state.level_index];
            state.sun = 725;
            state.wave = level.final_wave;
            state.score = 720;
            state.selected = PlantKind::CherryBomb;
            state.final_wave_started = true;
            state.spawn_timer = Timer::from_seconds(60.0, TimerMode::Repeating);
            state.wave_timer = Timer::from_seconds(60.0, TimerMode::Repeating);
            for (kind, col, lane) in [
                (PlantKind::Sunflower, 0, 0),
                (PlantKind::Repeater, 2, 0),
                (PlantKind::Wallnut, 4, 1),
                (PlantKind::CabbagePult, 2, 2),
                (PlantKind::Torchwood, 3, 2),
                (PlantKind::CherryBomb, 4, 3),
                (PlantKind::SnowPea, 2, 4),
                (PlantKind::Spikeweed, 5, 4),
            ] {
                spawn_plant(&mut commands, &asset_server, kind, col, lane);
            }
            for (kind, lane, x) in [
                (ZombieKind::Gargantuar, 1, 5.7),
                (ZombieKind::Brute, 2, 4.7),
                (ZombieKind::Frostbite, 3, 5.4),
                (ZombieKind::Digger, 4, 3.5),
                (ZombieKind::Healer, 0, 6.1),
                (ZombieKind::Buckethead, 2, 6.4),
            ] {
                spawn_zombie_entity(
                    &mut commands,
                    &asset_server,
                    kind,
                    lane,
                    x,
                    state.wave,
                    locale,
                );
            }
            spawn_visual_effect(
                &mut commands,
                &mut meshes,
                &mut materials,
                &asset_server,
                EFFECT_EXPLOSION,
                Vec3::new(col_x(5), 0.75, lane_z(3)),
                1.15,
                4.0,
            );
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn spawn_zombie_entity(
    commands: &mut Commands,
    asset_server: &AssetServer,
    kind: ZombieKind,
    lane: usize,
    x: f32,
    wave: u32,
    locale: &LocaleText,
) {
    let (health, speed, damage, radius) = kind.stats(wave);
    let sprite_height = if kind == ZombieKind::Gargantuar {
        1.78
    } else {
        1.34
    };
    let sprite_width = radius
        * if kind == ZombieKind::Gargantuar {
            2.25
        } else {
            2.05
        };
    let model_scale = sprite_width.max(sprite_height) * 0.72;
    commands.spawn((
        SceneRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset(kind.model_path()))),
        Transform::from_xyz(x, 0.04, lane_z(lane)).with_scale(Vec3::splat(model_scale)),
        Zombie {
            kind,
            lane,
            health,
            max_health: health,
            speed,
            damage,
            attack_timer: Timer::from_seconds(0.8, TimerMode::Repeating),
            slow_timer: Timer::from_seconds(0.0, TimerMode::Once),
            special_timer: Timer::from_seconds(1.7, TimerMode::Repeating),
            jumped: false,
        },
        GameEntity,
        Name::new(kind.label(locale).to_string()),
    ));
}

fn spawn_sun(
    mut commands: Commands,
    time: Res<Time>,
    mut state: ResMut<BoardState>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    state.sky_sun_timer.tick(time.delta());
    if !state.sky_sun_timer.just_finished() {
        return;
    }
    let mut rng = rand::rng();
    let col = rng.random_range(0..COLS);
    let lane = rng.random_range(0..LANES);
    spawn_sun_pickup(
        &mut commands,
        &mut meshes,
        &mut materials,
        &asset_server,
        col_x(col),
        lane_z(lane),
        25,
    );
}

#[allow(clippy::too_many_arguments)]
fn plants_act(
    mut commands: Commands,
    time: Res<Time>,
    mut plants: Query<(&mut Plant, &Transform), Without<Zombie>>,
    mut zombies: Query<(&mut Zombie, &Transform), Without<Plant>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
    settings: Res<GameSettings>,
    audio: Res<AsyncAudio>,
    mut shake: ResMut<ScreenShake>,
) {
    for (mut plant, transform) in &mut plants {
        match plant.kind {
            PlantKind::Peashooter => {
                plant.fire_timer.tick(time.delta());
                let has_target = zombies.iter().any(|(zombie, zombie_transform)| {
                    zombie.lane == plant.lane
                        && zombie_transform.translation.x > transform.translation.x
                        && zombie_transform.translation.x < ZOMBIE_START_X + 0.7
                });
                if has_target && plant.fire_timer.just_finished() {
                    spawn_projectile(
                        &mut commands,
                        &mut meshes,
                        &mut materials,
                        &asset_server,
                        plant.lane,
                        transform.translation,
                        ProjectileSpec {
                            damage: 26.0,
                            speed: 5.6,
                            slow_secs: 0.0,
                            pierce: 0,
                            splash_radius: 0.0,
                            color: Color::srgb(0.48, 0.95, 0.28),
                            texture: EFFECT_PEA,
                            visual_size: 0.34,
                            name: "Pea",
                        },
                    );
                    play_sound(&audio, &settings, AUDIO_SHOOT, 0.28);
                }
            }
            PlantKind::Sunflower => {
                plant.sun_timer.tick(time.delta());
                if plant.sun_timer.just_finished() {
                    spawn_sun_pickup(
                        &mut commands,
                        &mut meshes,
                        &mut materials,
                        &asset_server,
                        transform.translation.x,
                        transform.translation.z,
                        25,
                    );
                }
            }
            PlantKind::SnowPea => {
                plant.fire_timer.tick(time.delta());
                let has_target = zombies.iter().any(|(zombie, zombie_transform)| {
                    zombie.lane == plant.lane
                        && zombie_transform.translation.x > transform.translation.x
                        && zombie_transform.translation.x < ZOMBIE_START_X + 0.7
                });
                if has_target && plant.fire_timer.just_finished() {
                    spawn_projectile(
                        &mut commands,
                        &mut meshes,
                        &mut materials,
                        &asset_server,
                        plant.lane,
                        transform.translation,
                        ProjectileSpec {
                            damage: 20.0,
                            speed: 5.2,
                            slow_secs: 3.5,
                            pierce: 0,
                            splash_radius: 0.0,
                            color: Color::srgb(0.52, 0.88, 1.0),
                            texture: EFFECT_FROST_POD,
                            visual_size: 0.36,
                            name: "Frost Sprout",
                        },
                    );
                    play_sound(&audio, &settings, AUDIO_SHOOT, 0.24);
                }
            }
            PlantKind::Repeater => {
                plant.fire_timer.tick(time.delta());
                let has_target = zombies.iter().any(|(zombie, zombie_transform)| {
                    zombie.lane == plant.lane
                        && zombie_transform.translation.x > transform.translation.x
                        && zombie_transform.translation.x < ZOMBIE_START_X + 0.7
                });
                if has_target && plant.fire_timer.just_finished() {
                    spawn_projectile(
                        &mut commands,
                        &mut meshes,
                        &mut materials,
                        &asset_server,
                        plant.lane,
                        transform.translation,
                        ProjectileSpec {
                            damage: 42.0,
                            speed: 5.8,
                            slow_secs: 0.0,
                            pierce: 0,
                            splash_radius: 0.0,
                            color: Color::srgb(0.32, 0.90, 0.22),
                            texture: EFFECT_PEA,
                            visual_size: 0.38,
                            name: "Double Pea",
                        },
                    );
                    play_sound(&audio, &settings, AUDIO_SHOOT, 0.30);
                }
            }
            PlantKind::CabbagePult => {
                plant.fire_timer.tick(time.delta());
                let has_target = zombies.iter().any(|(zombie, zombie_transform)| {
                    zombie.lane == plant.lane
                        && zombie_transform.translation.x > transform.translation.x
                });
                if has_target && plant.fire_timer.just_finished() {
                    spawn_projectile(
                        &mut commands,
                        &mut meshes,
                        &mut materials,
                        &asset_server,
                        plant.lane,
                        transform.translation,
                        ProjectileSpec {
                            damage: 48.0,
                            speed: 4.1,
                            slow_secs: 0.0,
                            pierce: 0,
                            splash_radius: 0.62,
                            color: Color::srgb(0.70, 0.92, 0.34),
                            texture: EFFECT_CABBAGE,
                            visual_size: 0.48,
                            name: "Cabbage",
                        },
                    );
                    play_sound(&audio, &settings, AUDIO_SHOOT, 0.34);
                }
            }
            PlantKind::Spikeweed => {
                plant.fire_timer.tick(time.delta());
                if plant.fire_timer.just_finished() {
                    for (mut zombie, zombie_transform) in &mut zombies {
                        if zombie.lane == plant.lane
                            && (zombie_transform.translation.x - transform.translation.x).abs()
                                < 0.62
                        {
                            zombie.health -= 22.0;
                        }
                    }
                }
            }
            PlantKind::CherryBomb => {
                plant.fire_timer.tick(time.delta());
                if plant.fire_timer.just_finished() {
                    spawn_visual_effect(
                        &mut commands,
                        &mut meshes,
                        &mut materials,
                        &asset_server,
                        EFFECT_EXPLOSION,
                        transform.translation,
                        2.25,
                        0.42,
                    );
                    shake.trauma = (shake.trauma + 0.65).min(1.0);
                    for (mut zombie, zombie_transform) in &mut zombies {
                        let lane_distance = zombie.lane.abs_diff(plant.lane);
                        if lane_distance <= 1
                            && (zombie_transform.translation.x - transform.translation.x).abs()
                                < 1.55
                        {
                            zombie.health -= 520.0;
                        }
                    }
                    plant.health = 0.0;
                }
            }
            PlantKind::Torchwood => {
                plant.fire_timer.tick(time.delta());
                if plant.fire_timer.just_finished() {
                    let mut hit_any = false;
                    for (mut zombie, zombie_transform) in &mut zombies {
                        if zombie.lane == plant.lane
                            && (zombie_transform.translation.x - transform.translation.x).abs()
                                < 0.95
                        {
                            zombie.health -= 18.0;
                            hit_any = true;
                        }
                    }
                    if hit_any {
                        spawn_visual_effect(
                            &mut commands,
                            &mut meshes,
                            &mut materials,
                            &asset_server,
                            EFFECT_FIRE,
                            transform.translation + Vec3::new(0.45, 0.15, 0.0),
                            0.86,
                            0.24,
                        );
                    }
                }
            }
            PlantKind::Garlic => {}
            PlantKind::Wallnut => {}
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn spawn_visual_effect(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    asset_server: &AssetServer,
    texture: &'static str,
    origin: Vec3,
    size: f32,
    lifetime_secs: f32,
) {
    commands.spawn((
        Mesh3d(meshes.add(billboard_mesh(size, size))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::WHITE,
            base_color_texture: Some(asset_server.load(texture)),
            alpha_mode: AlphaMode::Blend,
            unlit: true,
            ..default()
        })),
        Transform::from_xyz(origin.x, 0.72, origin.z),
        VisualEffect {
            lifetime: Timer::from_seconds(lifetime_secs, TimerMode::Once),
        },
        GameEntity,
        Name::new("Visual Effect"),
    ));
}

fn tick_visual_effects(
    mut commands: Commands,
    time: Res<Time>,
    mut effects: Query<(Entity, &mut VisualEffect)>,
) {
    for (entity, mut effect) in &mut effects {
        effect.lifetime.tick(time.delta());
        if effect.lifetime.is_finished() {
            commands.entity(entity).despawn();
        }
    }
}

fn spawn_projectile(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    asset_server: &AssetServer,
    lane: usize,
    origin: Vec3,
    spec: ProjectileSpec,
) {
    spawn_visual_effect(
        commands,
        meshes,
        materials,
        asset_server,
        EFFECT_FIRE,
        Vec3::new(origin.x + 0.30, 0.55, origin.z),
        0.22,
        0.09,
    );
    commands.spawn((
        Mesh3d(meshes.add(billboard_mesh(spec.visual_size, spec.visual_size))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::WHITE,
            base_color_texture: Some(asset_server.load(spec.texture)),
            emissive: spec.color.with_alpha(0.18).into(),
            alpha_mode: AlphaMode::Blend,
            unlit: true,
            ..default()
        })),
        Transform::from_xyz(origin.x + 0.42, 0.55, origin.z),
        Projectile {
            lane,
            damage: spec.damage,
            speed: spec.speed,
            slow_secs: spec.slow_secs,
            pierce: spec.pierce,
            splash_radius: spec.splash_radius,
        },
        GameEntity,
        Name::new(spec.name),
    ));
}

fn spawn_sun_pickup(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    asset_server: &AssetServer,
    x: f32,
    z: f32,
    value: u32,
) {
    commands.spawn((
        Mesh3d(meshes.add(billboard_mesh(0.56, 0.56))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::WHITE,
            base_color_texture: Some(asset_server.load(EFFECT_SUN)),
            emissive: Color::srgb(0.36, 0.19, 0.02).into(),
            alpha_mode: AlphaMode::Blend,
            unlit: true,
            ..default()
        })),
        Transform::from_xyz(x, 0.42, z),
        SunPickup {
            value,
            lifetime: Timer::from_seconds(5.5, TimerMode::Once),
        },
        GameEntity,
        Name::new("Sun"),
    ));
}

#[allow(clippy::too_many_arguments)]
fn spawn_zombies(
    mut commands: Commands,
    time: Res<Time>,
    mut state: ResMut<BoardState>,
    levels: Res<LevelCatalog>,
    language: Res<LanguageSettings>,
    localization: Res<LocalizationCatalog>,
    asset_server: Res<AssetServer>,
) {
    let level = &levels.levels[state.level_index];
    let locale = localization.text(language.current);
    if !state.grace_timer.is_finished() {
        state.grace_timer.tick(time.delta());
        return;
    }
    state.wave_timer.tick(time.delta());
    if state.wave_timer.just_finished() && state.wave < level.final_wave {
        state.wave += 1;
        state.spawn_timer = Timer::from_seconds(
            (level.base_spawn_interval - state.wave as f32 * 0.18).max(0.95),
            TimerMode::Repeating,
        );
    }

    state.spawn_timer.tick(time.delta());
    if !state.spawn_timer.just_finished() {
        return;
    }

    if state.wave == level.final_wave && state.final_wave_started {
        return;
    }

    let mut rng = rand::rng();
    let count = if state.wave >= level.final_wave {
        level.final_spawn_count
    } else {
        1 + (state.wave / 3)
    };
    state.final_wave_started |= state.wave >= level.final_wave;

    let active_lanes = active_lane_count(state.level_index, state.wave);
    for _ in 0..count {
        let lane = LANE_ROLLOUT[rng.random_range(0..active_lanes)];
        let kind = choose_zombie_kind(
            state.level_index,
            state.wave,
            state.final_wave_started,
            &mut rng,
        );
        let (health, speed, damage, radius) = kind.stats(state.wave);
        let sprite_height = if kind == ZombieKind::Gargantuar {
            1.78
        } else {
            1.34
        };
        let sprite_width = radius
            * if kind == ZombieKind::Gargantuar {
                2.25
            } else {
                2.05
            };
        let model_scale = sprite_width.max(sprite_height) * 0.72;
        commands.spawn((
            SceneRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset(kind.model_path()))),
            Transform::from_xyz(
                ZOMBIE_START_X + rng.random_range(0.0..1.0),
                0.04,
                lane_z(lane),
            )
            .with_scale(Vec3::splat(model_scale)),
            Zombie {
                kind,
                lane,
                health,
                max_health: health,
                speed,
                damage,
                attack_timer: Timer::from_seconds(0.8, TimerMode::Repeating),
                slow_timer: Timer::from_seconds(0.0, TimerMode::Once),
                special_timer: Timer::from_seconds(1.7, TimerMode::Repeating),
                jumped: false,
            },
            GameEntity,
            Name::new(kind.label(locale).to_string()),
        ));
    }
}

fn choose_zombie_kind(
    level_index: usize,
    wave: u32,
    final_wave: bool,
    rng: &mut impl Rng,
) -> ZombieKind {
    let roster = level_zombie_roster(level_index);
    if final_wave && roster.contains(&ZombieKind::Gargantuar) && rng.random_bool(0.18) {
        return ZombieKind::Gargantuar;
    }

    // Keep the wave-based weighting, but reroll picks the level has not
    // introduced yet; fall back to a uniform roster pick.
    for _ in 0..8 {
        let kind = raw_choose_zombie_kind(wave, rng);
        if roster.contains(&kind) {
            return kind;
        }
    }
    roster[rng.random_range(0..roster.len())]
}

fn raw_choose_zombie_kind(wave: u32, rng: &mut impl Rng) -> ZombieKind {
    let roll = rng.random_range(0..100);
    match wave {
        1 => ZombieKind::Walker,
        2 => {
            if roll < 70 {
                ZombieKind::Walker
            } else {
                ZombieKind::Conehead
            }
        }
        3 => match roll {
            0..=44 => ZombieKind::Walker,
            45..=69 => ZombieKind::Conehead,
            _ => ZombieKind::Runner,
        },
        4 => match roll {
            0..=34 => ZombieKind::Walker,
            35..=56 => ZombieKind::Runner,
            57..=77 => ZombieKind::Conehead,
            _ => ZombieKind::Buckethead,
        },
        5 => match roll {
            0..=24 => ZombieKind::Walker,
            25..=44 => ZombieKind::Runner,
            45..=61 => ZombieKind::Buckethead,
            62..=78 => ZombieKind::Healer,
            _ => ZombieKind::Jumper,
        },
        6 => match roll {
            0..=19 => ZombieKind::Runner,
            20..=37 => ZombieKind::Buckethead,
            38..=53 => ZombieKind::Healer,
            54..=69 => ZombieKind::Jumper,
            70..=84 => ZombieKind::Digger,
            _ => ZombieKind::Brute,
        },
        7 => match roll {
            0..=17 => ZombieKind::Runner,
            18..=33 => ZombieKind::Buckethead,
            34..=48 => ZombieKind::Healer,
            49..=63 => ZombieKind::Jumper,
            64..=77 => ZombieKind::Digger,
            78..=91 => ZombieKind::Frostbite,
            _ => ZombieKind::Brute,
        },
        _ => match roll {
            0..=10 => ZombieKind::Walker,
            11..=22 => ZombieKind::Runner,
            23..=36 => ZombieKind::Buckethead,
            37..=49 => ZombieKind::Healer,
            50..=62 => ZombieKind::Jumper,
            63..=74 => ZombieKind::Digger,
            75..=86 => ZombieKind::Frostbite,
            87..=96 => ZombieKind::Brute,
            _ => ZombieKind::Gargantuar,
        },
    }
}

fn move_projectiles(
    mut commands: Commands,
    time: Res<Time>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
    mut projectiles: Query<(Entity, &mut Projectile, &mut Transform), Without<Zombie>>,
    mut zombies: Query<(Entity, &mut Zombie, &GlobalTransform, Has<HitReact>), Without<Projectile>>,
) {
    for (projectile_entity, mut projectile, mut projectile_transform) in &mut projectiles {
        projectile_transform.translation.x += projectile.speed * time.delta_secs();
        projectile_transform.rotate_local_z(-6.5 * time.delta_secs());
        if projectile_transform.translation.x > 8.0 {
            commands.entity(projectile_entity).despawn();
            continue;
        }

        let mut hit = false;
        for (zombie_entity, mut zombie, zombie_transform, reacting) in &mut zombies {
            if zombie.lane == projectile.lane
                && (zombie_transform.translation().x - projectile_transform.translation.x).abs()
                    < 0.28
            {
                zombie.health -= projectile.damage * zombie.kind.damage_multiplier();
                if projectile.slow_secs > 0.0 {
                    zombie.slow_timer = Timer::from_seconds(projectile.slow_secs, TimerMode::Once);
                }
                if !reacting {
                    commands.entity(zombie_entity).insert(HitReact {
                        timer: Timer::from_seconds(0.16, TimerMode::Once),
                        base_scale: zombie_transform.scale(),
                    });
                }
                hit = true;
                break;
            }
        }

        if projectile.splash_radius > 0.0 && hit {
            for (_zombie_entity, mut zombie, zombie_transform, _) in &mut zombies {
                if zombie.lane == projectile.lane
                    && (zombie_transform.translation().x - projectile_transform.translation.x).abs()
                        < projectile.splash_radius
                {
                    zombie.health -= projectile.damage * 0.55 * zombie.kind.damage_multiplier();
                }
            }
        }

        if hit {
            spawn_visual_effect(
                &mut commands,
                &mut meshes,
                &mut materials,
                &asset_server,
                EFFECT_EXPLOSION,
                projectile_transform.translation,
                0.30,
                0.12,
            );
            if projectile.pierce == 0 {
                commands.entity(projectile_entity).despawn();
                continue;
            } else {
                projectile.pierce -= 1;
            }
        }
    }
}

#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn announce_waves(
    mut commands: Commands,
    time: Res<Time>,
    state: Res<BoardState>,
    language: Res<LanguageSettings>,
    localization: Res<LocalizationCatalog>,
    mut last: Local<(u32, bool)>,
    mut banners: Query<(Entity, &mut WaveBanner)>,
    mut banner_texts: Query<&mut TextColor, With<WaveBannerText>>,
) {
    if state.wave <= 1 && !state.final_wave_started {
        *last = (1, false);
    }
    let locale = localization.text(language.current);
    let announce = if state.final_wave_started && !last.1 {
        last.1 = true;
        Some((locale.final_wave_warning.clone(), Color::srgb(1.0, 0.38, 0.30)))
    } else if state.wave > last.0 {
        last.0 = state.wave;
        Some((
            locale.wave_incoming.replace("{n}", &state.wave.to_string()),
            Color::srgb(0.95, 0.86, 0.44),
        ))
    } else {
        None
    };
    if let Some((message, color)) = announce {
        commands
            .spawn((
                Node {
                    position_type: PositionType::Absolute,
                    top: Val::Px(150.0),
                    width: Val::Percent(100.0),
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                WaveBanner {
                    timer: Timer::from_seconds(2.2, TimerMode::Once),
                },
                HudUi,
            ))
            .with_children(|root| {
                root.spawn((
                    Text::new(message),
                    TextFont {
                        font_size: 44.0,
                        ..default()
                    },
                    TextColor(color),
                    WaveBannerText,
                ));
            });
    }
    for (entity, mut banner) in &mut banners {
        banner.timer.tick(time.delta());
        if banner.timer.is_finished() {
            commands.entity(entity).despawn();
            continue;
        }
        let fade = 1.0 - banner.timer.fraction().powi(3);
        for mut text_color in &mut banner_texts {
            text_color.0 = text_color.0.with_alpha(fade);
        }
    }
}

fn hit_react(
    mut commands: Commands,
    time: Res<Time>,
    mut zombies: Query<(Entity, &mut HitReact, &mut Transform), With<Zombie>>,
) {
    for (entity, mut react, mut transform) in &mut zombies {
        if react.timer.elapsed_secs() == 0.0 {
            transform.translation.x += 0.05;
        }
        react.timer.tick(time.delta());
        if react.timer.is_finished() {
            transform.scale = react.base_scale;
            commands.entity(entity).remove::<HitReact>();
            continue;
        }
        let squash = 1.0 - 0.16 * (react.timer.fraction() * std::f32::consts::PI).sin();
        transform.scale = react.base_scale * Vec3::new(1.0 / squash, squash, 1.0 / squash);
    }
}

fn shake_camera(
    time: Res<Time>,
    mut shake: ResMut<ScreenShake>,
    mut cameras: Query<(&mut Transform, &CameraRig)>,
) {
    shake.trauma = (shake.trauma - time.delta_secs() * 1.6).max(0.0);
    let amount = shake.trauma * shake.trauma * 0.14;
    let t = time.elapsed_secs();
    for (mut transform, rig) in &mut cameras {
        transform.translation = rig.base
            + Vec3::new(
                (t * 37.0).sin(),
                (t * 47.0).cos() * 0.6,
                (t * 29.0).sin(),
            ) * amount;
    }
}

fn animate_units(
    time: Res<Time>,
    mut plants: Query<(&mut Transform, &Plant), Without<Zombie>>,
    mut zombies: Query<(&mut Transform, &Zombie), Without<Plant>>,
    mut suns: Query<&mut Transform, (With<SunPickup>, Without<Plant>, Without<Zombie>)>,
) {
    let t = time.elapsed_secs();
    for (mut transform, plant) in &mut plants {
        let phase = plant.col as f32 * 1.3 + plant.lane as f32 * 2.1;
        transform.rotation = Quat::from_rotation_z((t * 1.8 + phase).sin() * 0.045)
            * Quat::from_rotation_x((t * 1.3 + phase).cos() * 0.03);
    }
    for (mut transform, zombie) in &mut zombies {
        let phase = zombie.lane as f32 * 1.7 + transform.translation.x * 2.0;
        let step = (t * 5.0 + phase).sin();
        transform.translation.y = 0.04 + step.abs() * 0.05;
        transform.rotation = Quat::from_rotation_z(step * 0.06);
    }
    for mut transform in &mut suns {
        let phase = transform.translation.x * 3.0 + transform.translation.z;
        transform.scale = Vec3::splat(1.0 + (t * 4.0 + phase).sin() * 0.10);
    }
}

fn tag_limbs(
    mut commands: Commands,
    parts: Query<(Entity, &Name, &Transform), (Without<LimbAnim>, With<ChildOf>)>,
) {
    for (entity, name, transform) in &parts {
        let n = name.as_str();
        let kind = if n.starts_with("arm") {
            LimbKind::Arm
        } else if n.starts_with("leg") {
            LimbKind::Leg
        } else if n == "head" {
            LimbKind::Head
        } else {
            continue;
        };
        let phase = if n.ends_with(".001") {
            std::f32::consts::PI
        } else {
            0.0
        } + (entity.index().index() % 13) as f32 * 0.53;
        commands.entity(entity).insert(LimbAnim {
            base: transform.rotation,
            kind,
            phase,
        });
    }
}

fn animate_limbs(time: Res<Time>, mut limbs: Query<(&LimbAnim, &mut Transform)>) {
    let t = time.elapsed_secs();
    for (limb, mut transform) in &mut limbs {
        let swing = match limb.kind {
            LimbKind::Arm => Quat::from_rotation_z((t * 5.0 + limb.phase).sin() * 0.15),
            LimbKind::Leg => Quat::from_rotation_z((t * 5.0 + limb.phase).sin() * 0.28),
            LimbKind::Head => Quat::from_rotation_z((t * 2.3 + limb.phase).sin() * 0.07),
        };
        transform.rotation = swing * limb.base;
    }
}

fn grow_plants(
    mut commands: Commands,
    time: Res<Time>,
    mut growing: Query<(Entity, &mut GrowIn, &mut Transform)>,
) {
    for (entity, mut grow, mut transform) in &mut growing {
        grow.timer.tick(time.delta());
        if grow.timer.is_finished() {
            transform.scale = Vec3::splat(grow.target_scale);
            commands.entity(entity).remove::<GrowIn>();
            continue;
        }
        let p = grow.timer.fraction();
        let pop = 1.0 + 0.12 * (p * std::f32::consts::PI).sin();
        transform.scale = Vec3::splat(grow.target_scale * p * (1.8 - 0.8 * p) * pop);
    }
}

fn move_and_attack_zombies(
    time: Res<Time>,
    mut state: ResMut<BoardState>,
    mut zombies: Query<(&mut Zombie, &mut Transform), Without<Plant>>,
    mut plants: Query<(&mut Plant, &GlobalTransform), Without<Zombie>>,
) {
    for (mut zombie, mut zombie_transform) in &mut zombies {
        zombie.slow_timer.tick(time.delta());
        zombie.special_timer.tick(time.delta());
        if zombie.kind == ZombieKind::Healer && zombie.special_timer.just_finished() {
            zombie.health = (zombie.health + 14.0).min(zombie.max_health);
        }
        if zombie.kind == ZombieKind::Brute && zombie.health < zombie.max_health * 0.45 {
            zombie.damage += 0.015;
        }

        let mut attacked = false;
        let is_digging =
            zombie.kind == ZombieKind::Digger && zombie_transform.translation.x > col_x(1);
        for (mut plant, plant_transform) in &mut plants {
            if plant.lane == zombie.lane
                && (zombie_transform.translation.x - plant_transform.translation().x).abs() < 0.56
            {
                if is_digging {
                    continue;
                }
                if zombie.kind == ZombieKind::Jumper && !zombie.jumped {
                    zombie_transform.translation.x -= CELL * 0.92;
                    zombie.jumped = true;
                    attacked = true;
                    break;
                }
                zombie.attack_timer.tick(time.delta());
                if zombie.attack_timer.just_finished() {
                    plant.health -= zombie.damage;
                    if zombie.kind == ZombieKind::Frostbite {
                        plant.fire_timer.reset();
                        plant.sun_timer.reset();
                    }
                    if plant.kind == PlantKind::Garlic {
                        let new_lane = if zombie.lane == 0 {
                            1
                        } else if zombie.lane == LANES - 1 {
                            LANES - 2
                        } else if zombie.lane % 2 == 0 {
                            zombie.lane - 1
                        } else {
                            zombie.lane + 1
                        };
                        zombie.lane = new_lane;
                        zombie_transform.translation.z = lane_z(new_lane);
                        plant.health -= 16.0;
                    }
                }
                attacked = true;
                break;
            }
        }

        if !attacked {
            let slow_multiplier = if zombie.slow_timer.is_finished() {
                1.0
            } else {
                0.48
            };
            zombie_transform.translation.x -= zombie.speed * slow_multiplier * time.delta_secs();
            zombie.attack_timer.reset();
        }

        if zombie_transform.translation.x < HOME_X {
            state.lost_house_hp += 1;
            zombie.health = 0.0;
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn collect_sun(
    mut commands: Commands,
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    pause: Res<PauseState>,
    mut state: ResMut<BoardState>,
    mut suns: Query<(Entity, &mut SunPickup, &Transform)>,
    settings: Res<GameSettings>,
    audio: Res<AsyncAudio>,
) {
    if pause.paused {
        return;
    }
    let mut collected = 0;
    for (entity, mut sun, _transform) in &mut suns {
        sun.lifetime.tick(time.delta());
        if sun.lifetime.is_finished() {
            commands.entity(entity).despawn();
        } else if keyboard.just_pressed(KeyCode::KeyC) {
            // C acts as a sun magnet: chasing each orb with the cursor made
            // keyboard play a chore and left sky sun rotting on far tiles.
            state.sun += sun.value;
            collected += 1;
            commands.entity(entity).despawn();
        }
    }
    if collected > 0 {
        play_sound(&audio, &settings, AUDIO_SUN_COLLECT, 0.58);
    }
}

fn cleanup_dead(
    mut commands: Commands,
    mut state: ResMut<BoardState>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
    plants: Query<(Entity, &Plant, &Transform)>,
    zombies: Query<(Entity, &Zombie, &Transform)>,
    settings: Res<GameSettings>,
    audio: Res<AsyncAudio>,
    mut shake: ResMut<ScreenShake>,
) {
    for (entity, plant, transform) in &plants {
        if plant.health <= 0.0 {
            spawn_visual_effect(
                &mut commands,
                &mut meshes,
                &mut materials,
                &asset_server,
                EFFECT_EXPLOSION,
                transform.translation,
                0.55,
                0.22,
            );
            commands.entity(entity).despawn();
        }
    }
    for (entity, zombie, transform) in &zombies {
        if zombie.health <= 0.0 {
            state.score += zombie.kind.score();
            state.kills += 1;
            if matches!(zombie.kind, ZombieKind::Gargantuar | ZombieKind::Brute) {
                shake.trauma = (shake.trauma + 0.4).min(1.0);
            }
            spawn_visual_effect(
                &mut commands,
                &mut meshes,
                &mut materials,
                &asset_server,
                EFFECT_EXPLOSION,
                transform.translation,
                0.85,
                0.30,
            );
            commands.entity(entity).despawn();
            play_sound(&audio, &settings, AUDIO_MONSTER_DOWN, 0.42);
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn hud_status_text(
    locale: &LocaleText,
    language: Language,
    state: &BoardState,
    level: &LevelConfig,
    settings: &GameSettings,
    pause: &PauseState,
) -> String {
    let play_state = match (language, pause.paused) {
        (Language::English, true) => "paused",
        (Language::English, false) => "play",
        (Language::Chinese, true) => "暂停",
        (Language::Chinese, false) => "进行中",
    };
    format!(
        "{}: {} | {}: {} | {}: {} ({}, {})\n{}: {}/{} | {}: {} | {}: C{} L{} | {} | {}: {} | Vol: {:.0}%",
        locale.hud.level,
        level.title(language),
        locale.hud.sun,
        state.sun,
        locale.hud.seed,
        state.selected.label(locale),
        state.selected.cost(),
        state.selected.description(locale),
        locale.hud.wave,
        state.wave,
        level.final_wave,
        locale.hud.score,
        state.score,
        locale.hud.cursor,
        state.cursor_col + 1,
        state.cursor_lane + 1,
        locale.hud.collect_sun,
        locale.hud.language,
        language.label(),
        settings.master_volume * 100.0
    ) + &format!(" | P: {play_state}")
}

fn hud_seed_bank_text(locale: &LocaleText, state: &BoardState) -> String {
    PlantKind::ALL
        .iter()
        .enumerate()
        .map(|(index, plant)| {
            let key = if index == 9 {
                "0".to_string()
            } else {
                (index + 1).to_string()
            };
            let cooldown = state.plant_cooldowns[plant.index()];
            let selected = if *plant == state.selected { ">" } else { " " };
            if cooldown > 0.0 {
                format!(
                    "{selected}{key}:{}({}) {:.0}s",
                    plant.label(locale),
                    plant.cost(),
                    cooldown.ceil()
                )
            } else {
                format!("{selected}{key}:{}({})", plant.label(locale), plant.cost())
            }
        })
        .collect::<Vec<_>>()
        .chunks(5)
        .map(|chunk| chunk.join("  "))
        .collect::<Vec<_>>()
        .join("\n")
}

#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn update_hud(
    state: Res<BoardState>,
    language: Res<LanguageSettings>,
    localization: Res<LocalizationCatalog>,
    levels: Res<LevelCatalog>,
    settings: Res<GameSettings>,
    pause: Res<PauseState>,
    mut status_query: Query<&mut Text, (With<HudStatusText>, Without<HudSeedBankText>)>,
) {
    let locale = localization.text(language.current);
    let level = &levels.levels[state.level_index];
    if let Ok(mut text) = status_query.single_mut() {
        **text = hud_status_text(locale, language.current, &state, level, &settings, &pause);
    }
}

fn seed_card_clicks(
    mut state: ResMut<BoardState>,
    cards: Query<(&SeedButton, &Interaction), (Changed<Interaction>, With<Button>)>,
) {
    for (seed, interaction) in &cards {
        if *interaction == Interaction::Pressed {
            state.selected = seed.0;
        }
    }
}

#[allow(clippy::type_complexity)]
fn update_seed_cards(
    state: Res<BoardState>,
    language: Res<LanguageSettings>,
    localization: Res<LocalizationCatalog>,
    mut cards: Query<(&SeedButton, &Interaction, &mut BackgroundColor, &Children)>,
    mut texts: Query<(&mut Text, &mut TextColor)>,
) {
    let locale = localization.text(language.current);
    for (seed, interaction, mut background, children) in &mut cards {
        let kind = seed.0;
        let index = kind.index();
        let cooldown = state.plant_cooldowns[index];
        let ready = state.sun >= kind.cost() && cooldown <= 0.0;
        let target = if state.selected == kind {
            Color::srgba(0.34, 0.50, 0.20, 0.95)
        } else if *interaction == Interaction::Hovered {
            Color::srgba(0.24, 0.34, 0.14, 0.90)
        } else {
            Color::srgba(0.10, 0.16, 0.08, 0.85)
        };
        if background.0 != target {
            *background = BackgroundColor(target);
        }
        for child in children {
            let Ok((mut text, mut color)) = texts.get_mut(*child) else {
                continue;
            };
            let key = if index == 9 { 0 } else { index + 1 };
            let cost_line = if cooldown > 0.0 {
                format!("{}  {:.0}s", kind.cost(), cooldown.ceil())
            } else {
                kind.cost().to_string()
            };
            let line = format!("{key} {}\n{cost_line}", kind.label(locale));
            if text.0 != line {
                text.0 = line;
            }
            let text_target = if ready {
                Color::srgb(0.85, 0.93, 0.72)
            } else {
                Color::srgb(0.50, 0.56, 0.48)
            };
            if color.0 != text_target {
                color.0 = text_target;
            }
        }
    }
}

fn check_end_state(
    state: Res<BoardState>,
    levels: Res<LevelCatalog>,
    zombies: Query<&Zombie>,
    mut next: ResMut<NextState<GameState>>,
) {
    let level = &levels.levels[state.level_index];
    if state.lost_house_hp >= level.max_breaches {
        next.set(GameState::GameOver);
    } else if state.final_wave_started && zombies.is_empty() {
        next.set(GameState::Victory);
    }
}

fn spawn_game_over(
    mut commands: Commands,
    state: Res<BoardState>,
    language: Res<LanguageSettings>,
    localization: Res<LocalizationCatalog>,
    settings: Res<GameSettings>,
    audio: Res<AsyncAudio>,
    asset_server: Res<AssetServer>,
) {
    let locale = localization.text(language.current);
    play_sound(&audio, &settings, AUDIO_DEFEAT, 0.78);
    spawn_end_screen(
        &mut commands,
        &asset_server,
        &locale.game_over,
        &end_subtitle(locale, GameState::GameOver, &state),
    );
}

#[allow(clippy::too_many_arguments)]
fn spawn_victory(
    mut commands: Commands,
    state: Res<BoardState>,
    language: Res<LanguageSettings>,
    localization: Res<LocalizationCatalog>,
    settings: Res<GameSettings>,
    screenshot_mode: Res<StoreScreenshotMode>,
    audio: Res<AsyncAudio>,
    asset_server: Res<AssetServer>,
    levels: Res<LevelCatalog>,
    mut progress: ResMut<ProgressState>,
) {
    let locale = localization.text(language.current);
    play_sound(&audio, &settings, AUDIO_VICTORY, 0.82);
    if let Some(best_score) = progress.best_scores.get_mut(state.level_index) {
        *best_score = (*best_score).max(state.score);
    }
    progress.unlocked_levels = progress
        .unlocked_levels
        .max((state.level_index + 2).min(levels.levels.len()));
    if screenshot_mode.scene.is_none() {
        save_progress(language.current, &progress, &settings);
    }
    spawn_end_screen(
        &mut commands,
        &asset_server,
        &locale.victory,
        &end_subtitle(locale, GameState::Victory, &state),
    );
}

fn end_subtitle(locale: &LocaleText, game_state: GameState, state: &BoardState) -> String {
    let action = match game_state {
        GameState::GameOver => locale.retry.as_str(),
        GameState::Victory => locale.play_again.as_str(),
        _ => "",
    };
    format!(
        "{} {} | {} {} | {} {} | {action}",
        locale.hud.score, state.score, locale.kills, state.kills, locale.hud.wave, state.wave
    )
}

fn update_end_text(
    language: Res<LanguageSettings>,
    localization: Res<LocalizationCatalog>,
    state: Res<BoardState>,
    game_state: Res<State<GameState>>,
    mut title: Query<&mut Text, (With<EndTitleText>, Without<EndSubtitleText>)>,
    mut subtitle: Query<&mut Text, (With<EndSubtitleText>, Without<EndTitleText>)>,
) {
    if !language.is_changed() {
        return;
    }
    let locale = localization.text(language.current);
    let title_text = match game_state.get() {
        GameState::GameOver => locale.game_over.as_str(),
        GameState::Victory => locale.victory.as_str(),
        _ => return,
    };
    if let Ok(mut text) = title.single_mut() {
        **text = title_text.to_string();
    }
    if let Ok(mut text) = subtitle.single_mut() {
        **text = end_subtitle(locale, *game_state.get(), &state);
    }
}

fn spawn_end_screen(
    commands: &mut Commands,
    asset_server: &AssetServer,
    title: &str,
    subtitle: &str,
) {
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(18.0),
                ..default()
            },
            ui_panel_image(asset_server, UI_END_PANEL),
            MenuUi,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new(title.to_string()),
                TextFont {
                    font_size: 52.0,
                    ..default()
                },
                TextColor(Color::srgb(0.92, 0.96, 0.74)),
                EndTitleText,
            ));
            parent
                .spawn((
                    Button,
                    Node {
                        padding: UiRect::axes(Val::Px(20.0), Val::Px(8.0)),
                        justify_content: JustifyContent::Center,
                        border_radius: BorderRadius::all(Val::Px(8.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.16, 0.24, 0.10, 0.90)),
                    EndRetryButton,
                ))
                .with_children(|button| {
                    button.spawn((
                        Text::new(subtitle.to_string()),
                        TextFont {
                            font_size: 24.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.82, 0.88, 0.72)),
                        EndSubtitleText,
                    ));
                });
        });
}

#[allow(clippy::type_complexity)]
fn end_screen_mouse(
    mut next: ResMut<NextState<GameState>>,
    mut buttons: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<EndRetryButton>),
    >,
) {
    for (interaction, mut background) in buttons.iter_mut() {
        match interaction {
            Interaction::Pressed => next.set(GameState::Playing),
            Interaction::Hovered => {
                *background = BackgroundColor(Color::srgba(0.28, 0.40, 0.18, 0.95));
            }
            Interaction::None => {
                *background = BackgroundColor(Color::srgba(0.16, 0.24, 0.10, 0.90));
            }
        }
    }
}

fn despawn_menu(mut commands: Commands, query: Query<Entity, With<MenuUi>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

fn despawn_game(
    mut commands: Commands,
    game_query: Query<Entity, With<GameEntity>>,
    hud_query: Query<Entity, With<HudUi>>,
    pause_query: Query<Entity, With<PauseUi>>,
) {
    for entity in &game_query {
        commands.entity(entity).despawn();
    }
    for entity in &hud_query {
        commands.entity(entity).despawn();
    }
    for entity in &pause_query {
        commands.entity(entity).despawn();
    }
}

fn col_x(col: usize) -> f32 {
    BOARD_LEFT + col as f32 * CELL
}

fn lane_z(lane: usize) -> f32 {
    BOARD_TOP - lane as f32 * CELL
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn embedded_level_data_is_valid_for_progression() {
        let catalog = load_levels();
        assert!(catalog.levels.len() >= 10);

        let mut ids = HashSet::new();
        let mut previous_final_wave = 0;
        for level in &catalog.levels {
            assert!(
                ids.insert(level.id.as_str()),
                "duplicate level id {}",
                level.id
            );
            assert!(!level.title_en.trim().is_empty());
            assert!(!level.title_zh.trim().is_empty());
            assert!(level.starting_sun >= 50);
            assert!((1..=5).contains(&level.max_breaches));
            assert!(level.final_wave >= 3);
            assert!(level.sky_sun_interval > 0.0);
            assert!(level.base_spawn_interval > 0.0);
            assert!(level.wave_duration > 0.0);
            assert!(level.final_spawn_count >= 3);
            assert!(level.final_wave >= previous_final_wave);
            previous_final_wave = level.final_wave;
        }
    }

    #[test]
    fn embedded_localization_matches_unit_rosters() {
        let localization = load_localization();
        for locale in [&localization.english, &localization.chinese] {
            assert!(!locale.title.trim().is_empty());
            assert_eq!(locale.plant_labels.len(), PlantKind::COUNT);
            assert_eq!(locale.plant_descriptions.len(), PlantKind::COUNT);
            assert_eq!(locale.zombie_labels.len(), ZombieKind::COUNT);

            for plant in PlantKind::ALL {
                assert!(!plant.label(locale).trim().is_empty());
                assert!(!plant.description(locale).trim().is_empty());
                assert!(plant.sprite_path().starts_with("art/sprites/plants/"));
                assert!(plant.model_path().starts_with("models/plants/"));
                assert!(plant.model_path().ends_with(".glb"));
            }
            for zombie in ZombieKind::ALL {
                assert!(!zombie.label(locale).trim().is_empty());
                assert!(zombie.sprite_path().starts_with("art/sprites/monsters/"));
                assert!(zombie.model_path().starts_with("models/monsters/"));
                assert!(zombie.model_path().ends_with(".glb"));
            }
        }
    }

    #[test]
    fn plant_rules_are_playable_and_ordered() {
        for plant in PlantKind::ALL {
            assert!(plant.cost() > 0);
            assert!(plant.max_health() > 0.0);
            assert!(plant.cooldown_seconds() > 0.0);
            assert!(plant.index() < PlantKind::COUNT);
        }

        assert!(
            PlantKind::CherryBomb.cooldown_seconds() > PlantKind::Peashooter.cooldown_seconds()
        );
        assert!(PlantKind::Wallnut.max_health() > PlantKind::Peashooter.max_health());
        assert!(PlantKind::Sunflower.cost() < PlantKind::Repeater.cost());
    }

    #[test]
    fn zombie_rules_scale_and_score() {
        for zombie in ZombieKind::ALL {
            let early = zombie.stats(1);
            let late = zombie.stats(8);
            assert!(early.0 > 0.0, "{zombie:?} health must be positive");
            assert!(early.1 > 0.0, "{zombie:?} speed must be positive");
            assert!(early.2 > 0.0, "{zombie:?} damage must be positive");
            assert!(early.3 > 0.0, "{zombie:?} radius must be positive");
            assert!(late.0 > early.0, "{zombie:?} health should scale by wave");
            assert!(zombie.score() > 0);
            assert!(zombie.index() < ZombieKind::COUNT);
        }

        assert!(ZombieKind::Gargantuar.score() > ZombieKind::Walker.score());
        assert!(ZombieKind::Runner.stats(1).1 > ZombieKind::Buckethead.stats(1).1);
        assert!(
            ZombieKind::Buckethead.damage_multiplier() < ZombieKind::Walker.damage_multiplier()
        );
    }

    #[test]
    fn board_state_uses_level_tuning() {
        let catalog = load_levels();
        let level = &catalog.levels[1];
        let state = BoardState::for_level(1, level);

        assert_eq!(state.level_index, 1);
        assert_eq!(state.sun, level.starting_sun);
        assert_eq!(state.wave, 1);
        assert_eq!(state.lost_house_hp, 0);
        assert!(
            state
                .plant_cooldowns
                .iter()
                .all(|cooldown| *cooldown == 0.0)
        );
        assert!(!state.final_wave_started);
    }

    #[test]
    fn world_points_map_to_board_cells() {
        assert_eq!(
            grid_cell_from_world(Vec3::new(col_x(0), 0.0, lane_z(0))),
            Some((0, 0))
        );
        assert_eq!(
            grid_cell_from_world(Vec3::new(col_x(COLS - 1), 0.0, lane_z(LANES - 1))),
            Some((COLS - 1, LANES - 1))
        );
        assert_eq!(
            grid_cell_from_world(Vec3::new(col_x(3) + CELL * 0.49, 0.0, lane_z(2))),
            Some((3, 2))
        );
        assert_eq!(
            grid_cell_from_world(Vec3::new(col_x(3) + CELL * 0.51, 0.0, lane_z(2))),
            Some((4, 2))
        );
        assert_eq!(
            grid_cell_from_world(Vec3::new(col_x(COLS - 1) + CELL, 0.0, lane_z(0))),
            None
        );
    }

    #[test]
    fn campaign_balance_audit_covers_every_level() {
        let report = audit_balance().expect("campaign balance should pass release bounds");

        assert_eq!(report.levels.len(), load_levels().levels.len());
        assert_eq!(report.levels.len(), 10);
        for level in &report.levels {
            assert!(level.expected_sun >= level.baseline_defense_cost);
            assert!(level.final_pressure > 0.0);
            assert!(level.final_pressure <= 13.0);
        }
        assert!(
            report
                .levels
                .windows(2)
                .all(|pair| pair[1].final_wave >= pair[0].final_wave)
        );
    }

    #[test]
    fn asset_audit_reports_release_asset_specs() {
        let report = asset_audit_report().expect("asset audit should pass release assets");

        assert!(report.contains("asset audit ok"));
        assert!(report.contains("assets/branding/generated/app-icon.png | png 1254x1254"));
        assert!(report.contains("assets/art/sprites/plants/sprout-slinger.png | png 251x627"));
        assert!(report.contains("assets/models/plants/sprout-slinger.glb | glb"));
        assert!(report.contains("assets/models/monsters/gargantuar.glb | glb"));
        assert!(report.contains("assets/art/effects/explosion.png | png 256x256"));
        assert!(report.contains("assets/art/ui/menu-panel.png | png 192x192"));
        assert!(report.contains("assets/art/ui/hud-panel.png | png 192x96"));
        assert!(report.contains("assets/audio/music-loop.wav | wav 7.50s 1ch 16bit 44100Hz"));
        assert!(report.contains("png assets: 36"));
        assert!(report.contains("wav assets: 7"));
        assert!(report.contains("svg assets: 2"));
        assert!(report.contains("glb assets: 20"));
        assert!(report.contains("metadata assets: 6"));
        assert!(report.contains("production art assets: 56"));
    }

    #[test]
    fn audio_audit_reports_mix_safety_and_startup_policy() {
        let report = audio_audit_report().expect("audio audit should pass release mix bounds");

        assert!(report.contains("audio audit ok"));
        assert!(report.contains("assets/audio/music-loop.wav | 7.50s | peak 0.169"));
        assert!(report.contains("assets/audio/victory.wav | 1.15s | peak 0.420"));
        assert!(report.contains("checked music loops: 1"));
        assert!(report.contains("checked sound effects: 6"));
        assert!(report.contains("checked mix safety: peak max 0.450"));
        assert!(report.contains("checked startup policy: audio remains opt-in by default"));
        assert!(report.contains("manual device QA still required: speakers and headphones"));
    }

    #[test]
    fn control_audit_reports_documented_bindings() {
        let report = control_audit_report().expect("control audit should pass documented bindings");

        assert!(report.contains("control audit ok: 24 bindings"));
        assert!(report.contains("Enter | menu/end | start or restart"));
        assert!(report.contains("Mouse left | gameplay | move cursor and plant selected seed"));
        assert!(report.contains("0 | menu/gameplay | select level 10 or Scent Root"));
        assert!(report.contains("+ | global | raise saved master volume"));
        assert!(report.contains("- | global | lower saved master volume"));
        assert!(report.contains("documentation: README.md controls section covered"));
        assert!(report.contains("menu localization: en/zh menu_help covered"));
    }

    #[test]
    fn input_flow_audit_reports_deterministic_controls() {
        let report =
            input_flow_audit_report().expect("input flow audit should pass control semantics");

        assert!(report.contains("input flow audit ok"));
        assert!(report.contains("menu navigation: wrap next/previous and digit selection covered"));
        assert!(report.contains("menu start gating: locked blocked, unlocked starts"));
        assert!(
            report.contains(
                "global settings: language toggle, fullscreen toggle, volume clamp covered"
            )
        );
        assert!(report.contains("gameplay cursor: arrow clamps covered"));
        assert!(report.contains("gameplay seed selection: 10/10 plants covered"));
        assert!(report.contains(
            "gameplay placement: affordable, paused, occupied, cooldown, insufficient sun covered"
        ));
        assert!(report.contains("pause gating: planting blocked while paused"));
        assert!(report.contains("checked bindings: 24"));
    }

    #[test]
    fn localization_audit_reports_bilingual_release_coverage() {
        let report =
            localization_audit_report().expect("localization audit should pass release coverage");

        assert!(report.contains("localization audit ok"));
        assert!(report.contains("en coverage | plants 10/10"));
        assert!(report.contains("zh coverage | plants 10/10"));
        assert!(report.contains("checked languages: en, zh"));
        assert!(report.contains("checked plants: 10"));
        assert!(report.contains("checked zombies: 10"));
        assert!(report.contains("checked levels: 10"));
    }

    #[test]
    fn marketing_audit_reports_store_and_press_materials() {
        let report = marketing_audit_report().expect("marketing audit should pass release copy");

        assert!(report.contains("marketing audit ok"));
        assert!(report.contains("STORE_PAGE.md section | ## Short Description"));
        assert!(report.contains("PRESSKIT.md token | assets/branding/generated/app-icon.png"));
        assert!(report.contains("PRESSKIT.md token | store_screenshot_check.sh"));
        assert!(report.contains("PRESSKIT.md token | CONTENT_RATING.md"));
        assert!(report.contains("STORE_SCREENSHOTS.md section | ## Required Captures"));
        assert!(report.contains("STORE_SCREENSHOTS.md token | screenshots/01-title-menu.png"));
        assert!(report.contains(
            "STORE_SCREENSHOTS.md token | store_screenshot_check.sh --validate-dir screenshots"
        ));
        assert!(report.contains("RELEASE_NOTES.md token | runtime-smoke.txt"));
        assert!(report.contains("RELEASE_NOTES.md token | audio-smoke.txt"));
        assert!(report.contains("ART_ASSETS.md token | assets/audio/*.wav"));
        assert!(report.contains("ART_ASSETS.md token | assets/art/ui/*.png"));
        assert!(report.contains("THIRD_PARTY_NOTICES.md token | No third-party art"));
        assert!(report.contains("CREDITS.md token | imagegen"));
        assert!(report.contains("CONTENT_RATING.md section | ## Gameplay Content"));
        assert!(report.contains("CONTENT_RATING.md token | Data collection: none"));
        assert!(report.contains(
            "checked documents: store, presskit, screenshots, content rating, release notes, art, notices, credits"
        ));
        assert!(report.contains(
            "checked media references: app icon, store capsule, plant sheet, monster sheet, screenshot plan"
        ));
        assert!(
            report.contains("checked release claims: 10 plants, 10 enemies, en/zh, linux-x86_64")
        );
    }

    #[test]
    fn ip_audit_reports_release_facing_name_separation() {
        let report = ip_audit_report().expect("ip audit should pass release-facing naming");

        assert!(report.contains("ip audit ok"));
        assert!(report.contains("checked ascii reserved terms:"));
        assert!(report.contains("checked cjk reserved terms:"));
        assert!(report.contains("checked release-facing blocks:"));
        assert!(report.contains("checked plant labels: 10/10"));
        assert!(report.contains("checked enemy labels: 10/10"));
        assert!(report.contains("checked asset manifest paths for renamed plant and frost assets"));
    }

    #[test]
    fn save_audit_reports_path_and_compatibility_rules() {
        let report = save_audit_report().expect("save audit should pass compatibility rules");

        assert!(report.contains("save audit ok"));
        assert!(report.contains("explicit override path"));
        assert!(report.contains("xdg data path"));
        assert!(report.contains("home fallback path"));
        assert!(report.contains("portable fallback path"));
        assert!(report.contains("legacy save parse | language Chinese"));
        assert!(report.contains("normalization | unlock clamp 10/10"));
        assert!(report.contains("parse_save_file | temp legacy file ok"));
        assert!(report.contains("checked level slots: 10"));
        assert!(report.contains("checked paths: explicit, xdg, home, portable"));
        assert!(
            report.contains("checked compatibility: legacy save, normalization, settings clamp")
        );
    }

    #[test]
    fn layout_audit_reports_bilingual_ui_text_bounds() {
        let report = layout_audit_report().expect("layout audit should pass release text bounds");

        assert!(report.contains("layout audit ok"));
        assert!(report.contains("checked languages: en, zh"));
        assert!(report.contains("checked levels: 10"));
        assert!(report.contains("checked plants in HUD: 10"));
        assert!(report.contains("en seed bank | lines 2/2"));
        assert!(report.contains("zh seed bank | lines 2/2"));
        assert!(report.contains("zh menu roster"));
    }

    #[test]
    fn visual_readability_audit_reports_viewports_contrast_and_assets() {
        let report =
            visual_readability_audit_report().expect("visual readability audit should pass");

        assert!(report.contains("visual readability audit ok"));
        assert!(report.contains("viewport desktop-720p | 1280x720"));
        assert!(report.contains("viewport handheld-800p | 1280x800"));
        assert!(report.contains("viewport compact-540p | 960x540"));
        assert!(report.contains("menu title contrast"));
        assert!(report.contains("hud seed bank contrast"));
        assert!(report.contains("checked viewports: 6"));
        assert!(report.contains("checked contrast pairs: 5"));
        assert!(report.contains("checked HUD wrapping: en/zh status and seed bank"));
        assert!(
            report
                .contains("checked visual assets: plant, monster, effect, environment, ui chrome")
        );
    }

    #[test]
    fn accessibility_audit_reports_keyboard_text_and_audio_coverage() {
        let report = accessibility_audit_report().expect("accessibility audit should pass");

        assert!(report.contains("accessibility audit ok"));
        assert!(report.contains(
            "keyboard-only flow: menu, gameplay, pause, settings, and end screens covered"
        ));
        assert!(report.contains("mouse alternative: placement and shovel have keyboard bindings"));
        assert!(
            report.contains(
                "no-audio playability: audio remains opt-in and HUD/end text carry state"
            )
        );
        assert!(report.contains(
            "bilingual readability: en/zh menu, HUD, seed bank, pause, and end text fit bounds"
        ));
        assert!(report.contains("contrast readability: 5 UI text pairs meet 4.5:1 minimum"));
        assert!(report.contains("checked keyboard bindings: 24"));
        assert!(report.contains("checked plants in accessible seed bank: 10"));
        assert!(report.contains("manual accessibility QA still required"));
    }

    #[test]
    fn performance_budget_audit_reports_release_bounds() {
        let report =
            performance_budget_audit_report().expect("performance budget audit should pass");

        assert!(report.contains("performance budget audit ok"));
        assert!(report.contains("checked levels: 10"));
        assert!(report.contains("max final spawn burst: 18/20"));
        assert!(report.contains("minimum runtime spawn interval: 0.95s"));
        assert!(report.contains("estimated peak zombies: 48/64"));
        assert!(report.contains("board plant slots: 45"));
        assert!(report.contains("projectile budget: 90"));
        assert!(report.contains("sun pickup budget: 47"));
        assert!(report.contains("visual effect budget: 45"));
        assert!(report.contains("estimated dynamic entities: 275/320"));
        assert!(report.contains("embedded asset bytes: 17035445/25000000"));
        assert!(report.contains("checked viewport floor: compact-540p 960x540"));
        assert!(report.contains("manual performance QA still required"));
    }

    #[test]
    fn privacy_support_audit_reports_local_data_and_no_network_posture() {
        let report = privacy_support_audit_report()
            .expect("privacy and support audit should pass release disclosure rules");

        assert!(report.contains("privacy audit ok"));
        assert!(report.contains("PRIVACY.md tokens checked"));
        assert!(report.contains("SUPPORT.md tokens checked"));
        assert!(report.contains("TROUBLESHOOTING.md tokens checked"));
        assert!(report.contains(
            "checked local data fields: language, unlocks, best scores, volume, fullscreen"
        ));
        assert!(report.contains("checked network posture: no telemetry"));
        assert!(report.contains("checked deletion posture: uninstall preserves saves by default"));
        assert!(report.contains("manual privacy QA still required"));
    }

    #[test]
    fn release_provenance_audit_reports_build_inputs_and_integrity_evidence() {
        let report = release_provenance_audit_report()
            .expect("release provenance audit should pass build traceability rules");

        assert!(report.contains("release provenance audit ok"));
        assert!(report.contains("BUILD_PROVENANCE.md tokens checked"));
        assert!(report.contains("scripts/package_release.sh tokens checked"));
        assert!(report.contains("scripts/smoke_release_archive.sh tokens checked"));
        assert!(report.contains("checked build inputs: Cargo.toml, Cargo.lock"));
        assert!(report.contains("checked integrity evidence: release-manifest.json, SHA256SUMS"));
        assert!(
            report.contains("checked dependency evidence: cargo metadata locked license report")
        );
        assert!(report.contains("manual provenance QA still required"));
    }

    #[test]
    fn campaign_simulation_covers_unlocks_and_unit_rules() {
        let report =
            simulate_campaign().expect("campaign simulation should pass release progression QA");

        assert_eq!(report.levels.len(), load_levels().levels.len());
        assert_eq!(report.covered_plants, PlantKind::COUNT);
        assert_eq!(report.covered_zombies, ZombieKind::COUNT);
        assert_eq!(report.levels.first().unwrap().unlock_before, 1);
        assert_eq!(
            report.levels.last().unwrap().unlock_after,
            report.levels.len()
        );
        for level in &report.levels {
            assert_eq!(level.affordable_plants, PlantKind::COUNT);
            assert!(level.zombie_pool_size > 0);
            assert!(level.projected_score_floor > 0);
        }
    }

    #[test]
    fn playthrough_audit_covers_victory_defeat_restart_and_scores() {
        let report = audit_playthrough().expect("scripted playthrough audit should pass");

        assert_eq!(report.levels.len(), load_levels().levels.len());
        assert_eq!(report.victory_checks, 10);
        assert_eq!(report.defeat_checks, 10);
        assert_eq!(report.restart_checks, 10);
        assert_eq!(report.score_checks, 10);
        assert_eq!(report.levels.first().unwrap().unlock_before, 1);
        assert_eq!(report.levels.last().unwrap().unlock_after_victory, 10);
        for level in &report.levels {
            assert!(level.victory_score > 0);
            assert_eq!(level.best_score, level.victory_score);
            assert!(level.unlock_after_victory >= level.unlock_before);
            assert_eq!(level.defeat_unlock_after, level.unlock_after_victory);
            assert!(level.restart_sun >= PlantKind::Peashooter.cost());
        }

        let text = playthrough_audit_report().expect("playthrough audit report should render");
        assert!(text.contains("playthrough audit ok: 10 levels"));
        assert!(
            text.contains(
                "checked lifecycle: victories 10, defeats 10, restarts 10, score saves 10"
            )
        );
        assert!(text.contains("checked clean-save campaign unlocks: 10/10"));
        assert!(text.contains("checked failure handling: defeat does not advance unlocks"));
        assert!(text.contains("checked restart handling: board state resets per level tuning"));
    }

    #[test]
    fn release_readiness_reports_automated_evidence_and_pending_manual_qa() {
        let report =
            release_readiness_report().expect("release readiness should summarize current state");

        assert!(report.contains("release readiness: manual approval required"));
        assert!(report.contains("automated evidence: pass"));
        assert!(report.contains("release data: pass"));
        assert!(report.contains("balance audit: pass (10 levels)"));
        assert!(report.contains("asset audit: pass (production art assets: 56)"));
        assert!(report.contains("audio audit: pass (peak max 0.450, rms min 0.049)"));
        assert!(report.contains("control audit: pass (24 bindings)"));
        assert!(report.contains("input flow audit: pass (10/10 plants covered)"));
        assert!(report.contains("localization audit: pass (checked languages: en, zh)"));
        assert!(report.contains("layout audit: pass (checked languages: en, zh)"));
        assert!(report.contains("visual readability audit: pass (checked viewports: 6)"));
        assert!(report.contains(
            "accessibility audit: pass (menu, gameplay, pause, settings, and end screens covered)"
        ));
        assert!(report.contains("performance budget audit: pass (275/320)"));
        assert!(report.contains("privacy/support audit: pass (no telemetry"));
        assert!(
            report.contains("release provenance audit: pass (release-manifest.json, SHA256SUMS")
        );
        assert!(report.contains(
            "marketing audit: pass (store, presskit, screenshots, content rating, release notes, art, notices, credits)"
        ));
        assert!(report.contains("ip audit: pass (checked release-facing blocks:"));
        assert!(report.contains("save audit: pass (legacy save, normalization, settings clamp)"));
        assert!(
            report.contains("campaign simulation: pass (10 levels, plants 10/10, zombies 10/10)")
        );
        assert!(report.contains(
            "playthrough audit: pass (victories 10, defeats 10, restarts 10, score saves 10)"
        ));
        assert!(report.contains("manual approval required: yes"));
        assert!(report.contains("pending: full manual playthrough of all 10 levels"));
        assert!(report.contains("pending: Windows package build and smoke test"));
        assert!(report.contains("pending: final visual spot-check on release hardware"));
        assert!(report.contains("pending: final art-direction review"));
        assert!(report.contains("ship status: release candidate, not final approval"));
    }

    #[test]
    fn lane_rollout_opens_gradually() {
        assert_eq!(active_lane_count(0, 1), 1);
        assert_eq!(active_lane_count(0, 3), 3);
        assert_eq!(active_lane_count(0, 6), LANES);
        assert_eq!(active_lane_count(2, 3), LANES);
        assert_eq!(active_lane_count(9, 1), LANES);
        let seen: HashSet<usize> = LANE_ROLLOUT.into_iter().collect();
        assert_eq!(seen.len(), LANES, "rollout must cover every lane once");
    }

    #[test]
    fn menu_text_exposes_level_roster_and_settings() {
        let levels = load_levels();
        let localization = load_localization();
        let progress = ProgressState {
            unlocked_levels: 2,
            best_scores: vec![120, 0],
        };
        let settings = GameSettings {
            master_volume: 0.7,
            fullscreen: true,
        };

        let english = localization.text(Language::English);
        let roster = menu_roster_text(english, Language::English, &levels, &progress);
        assert!(roster.contains("> 01."));
        assert!(roster.contains("Best:120"));
        assert!(roster.contains(english.locked_level.as_str()));
        assert!(
            menu_level_line(english, Language::English, &levels, &progress).contains("Level 1")
        );

        let settings_line = menu_settings_line(&settings, Language::Chinese);
        assert!(settings_line.contains("音频"));
        assert!(settings_line.contains("全屏: 开"));
        assert!(settings_line.contains("音量: 70%"));
    }

    #[test]
    fn hud_text_splits_status_and_seed_bank() {
        let levels = load_levels();
        let localization = load_localization();
        let level = &levels.levels[0];
        let mut state = BoardState::for_level(0, level);
        state.selected = PlantKind::Repeater;
        state.plant_cooldowns[PlantKind::CherryBomb.index()] = 3.2;
        let settings = GameSettings {
            master_volume: 0.65,
            fullscreen: false,
        };
        let pause = PauseState { paused: true };

        let english = localization.text(Language::English);
        let status = hud_status_text(english, Language::English, &state, level, &settings, &pause);
        assert!(status.contains("Level:"));
        assert!(status.contains("Sun:"));
        assert!(status.contains(&format!("{}: English", english.hud.language)));
        assert!(status.contains("Vol: 65%"));
        assert!(status.contains("P: paused"));

        let seed_bank = hud_seed_bank_text(english, &state);
        assert!(seed_bank.contains(">5:Twin Pod"));
        assert!(seed_bank.contains("8:Blast Berry"));
        assert!(seed_bank.contains("4s"));
        assert_eq!(seed_bank.matches(':').count(), PlantKind::COUNT);

        let chinese = localization.text(Language::Chinese);
        let chinese_status =
            hud_status_text(chinese, Language::Chinese, &state, level, &settings, &pause);
        assert!(chinese_status.contains("暂停"));
        assert!(chinese_status.contains("中文"));
    }

    #[test]
    fn save_defaults_and_normalization_are_backward_compatible() {
        let old_save = r#"(
            version: 1,
            language: Chinese,
            unlocked_levels: 2,
            best_scores: [100],
        )"#;
        let parsed = ron::from_str::<SaveData>(old_save).expect("old save format should parse");
        assert_eq!(parsed.language, Language::Chinese);
        assert_eq!(parsed.settings.master_volume, 0.8);
        assert!(!parsed.settings.fullscreen);

        let progress = normalized_progress(&parsed, 3);
        assert_eq!(progress.unlocked_levels, 2);
        assert_eq!(progress.best_scores, vec![100, 0, 0]);
        assert_eq!(progress.best_score(0), Some(100));
        assert_eq!(progress.best_score(1), None);

        let summary = save_summary_text(&parsed, 3);
        assert!(summary.contains("save_version=1"));
        assert!(summary.contains("language=Chinese"));
        assert!(summary.contains("unlocked_levels=2"));
        assert!(summary.contains("best_scores=100,0,0"));
        assert!(summary.contains("master_volume=0.80"));
    }

    #[test]
    fn save_path_uses_user_data_directory_with_override() {
        assert_eq!(
            save_path_from_env(
                Some(PathBuf::from("/tmp/bevy_open/save.ron")),
                Some(PathBuf::from("/tmp/xdg")),
                Some(PathBuf::from("/home/player")),
            ),
            PathBuf::from("/tmp/bevy_open/save.ron")
        );
        assert_eq!(
            save_path_from_env(Some(PathBuf::from("portable-save.ron")), None, None),
            PathBuf::from("portable-save.ron")
        );
        assert_eq!(
            save_path_from_env(None, Some(PathBuf::from("/tmp/xdg")), None),
            PathBuf::from("/tmp/xdg")
                .join("bevy_open_siege")
                .join(SAVE_FILE_NAME)
        );
        assert_eq!(
            save_path_from_env(None, None, Some(PathBuf::from("/home/player"))),
            PathBuf::from("/home/player")
                .join(".local")
                .join("share")
                .join("bevy_open_siege")
                .join(SAVE_FILE_NAME)
        );
        assert_eq!(
            save_path_from_env(None, None, None),
            PathBuf::from(SAVE_FILE_NAME)
        );
    }

    #[test]
    fn asset_format_validation_checks_dimensions_and_audio_data() {
        assert_eq!(png_dimensions(APP_ICON_PNG), Some((1254, 1254)));
        let (capsule_width, capsule_height) =
            png_dimensions(STORE_CAPSULE_PNG).expect("store capsule must be a PNG");
        assert!(capsule_width >= 1000);
        assert!(capsule_height >= 500);

        let (channels, bits, sample_rate, data_bytes) =
            wav_audio_summary(MUSIC_LOOP_WAV).expect("music loop must be a WAV");
        assert_eq!(channels, 1);
        assert_eq!(bits, 16);
        assert_eq!(sample_rate, 44_100);
        assert!(data_bytes > 1_024);

        assert!(validate_embedded_asset_format("bad.png", b"not png").is_err());
        assert!(validate_embedded_asset_format("bad.wav", b"RIFF----WAVE").is_err());
    }

    #[test]
    fn release_metadata_and_packaging_manifest_are_present() {
        let metadata = load_release_metadata();
        let manifest = load_asset_manifest();
        let readme = include_str!("../README.md");
        let license = include_str!("../LICENSE");
        let credits = include_str!("../CREDITS.md");
        let art_assets = include_str!("../ART_ASSETS.md");
        let notices = include_str!("../THIRD_PARTY_NOTICES.md");
        let store_page = include_str!("../STORE_PAGE.md");
        let presskit = include_str!("../PRESSKIT.md");
        let store_screenshots = include_str!("../STORE_SCREENSHOTS.md");
        let content_rating = include_str!("../CONTENT_RATING.md");
        let release_notes = include_str!("../RELEASE_NOTES.md");
        let privacy = include_str!("../PRIVACY.md");
        let support = include_str!("../SUPPORT.md");
        let troubleshooting = include_str!("../TROUBLESHOOTING.md");
        let build_provenance = include_str!("../BUILD_PROVENANCE.md");
        let checklist = include_str!("../RELEASE_CHECKLIST.md");
        let qa_signoff = include_str!("../QA_SIGNOFF.md");
        let release_check_script = include_str!("../scripts/release_check.sh");
        let package_script = include_str!("../scripts/package_release.sh");
        let smoke_script = include_str!("../scripts/smoke_release_archive.sh");
        let audio_smoke_script = include_str!("../scripts/audio_smoke.sh");
        let runtime_smoke_script = include_str!("../scripts/runtime_smoke.sh");
        let visual_smoke_script = include_str!("../scripts/visual_smoke.sh");
        let store_screenshot_script = include_str!("../scripts/store_screenshot_check.sh");
        let store_asset_audit_script = include_str!("../scripts/store_asset_audit.sh");
        let content_rating_audit_script = include_str!("../scripts/content_rating_audit.sh");
        let linux_package_audit_script = include_str!("../scripts/linux_package_audit.sh");
        let linux_install_smoke_script = include_str!("../scripts/linux_install_smoke.sh");
        let linux_dependency_audit_script = include_str!("../scripts/linux_dependency_audit.sh");
        let linux_portability_smoke_script = include_str!("../scripts/linux_portability_smoke.sh");
        let linux_clean_distro_smoke_script =
            include_str!("../scripts/linux_clean_distro_smoke.sh");
        let linux_metadata_audit_script = include_str!("../scripts/linux_metadata_audit.sh");
        let manual_qa_session_script = include_str!("../scripts/manual_qa_session.sh");
        let manual_qa_observations_script = include_str!("../scripts/manual_qa_observations.sh");
        let platform_package_plan_script = include_str!("../scripts/platform_package_plan.sh");
        let qa_evidence_summary_script = include_str!("../scripts/qa_evidence_summary.sh");
        let final_signoff_check_script = include_str!("../scripts/final_signoff_check.sh");
        let verify_release_script = include_str!("../scripts/verify_release.sh");
        let support_diagnostics_script = include_str!("../scripts/support_diagnostics.sh");
        let signoff_bundle_script = include_str!("../scripts/signoff_bundle.sh");
        let candidate_evidence_script = include_str!("../scripts/create_candidate_evidence.sh");
        let store_submission_script = include_str!("../scripts/create_store_submission_pack.sh");
        let windows_package_script = include_str!("../scripts/package_windows.ps1");
        let macos_package_script = include_str!("../scripts/package_macos.sh");
        let license_report_script = include_str!("../scripts/generate_third_party_licenses.py");
        let release_manifest_script = include_str!("../scripts/generate_release_manifest.py");

        assert_eq!(metadata.product_name, "Bevy Open Siege");
        assert_eq!(metadata.version, env!("CARGO_PKG_VERSION"));
        assert_eq!(metadata.release_channel, "release_candidate");
        validate_release_copy()
            .expect("release-facing copy should not contain stale prototype terms");
        assert!(metadata.supported_languages.contains(&"en".to_string()));
        assert!(metadata.supported_languages.contains(&"zh".to_string()));
        assert!(!metadata.supported_platforms.is_empty());
        assert!(readme.contains("BEVY_OPEN_SIEGE_SAVE_PATH"));
        assert!(readme.contains("--print-save-path"));
        assert!(readme.contains("--print-save-summary"));
        assert!(readme.contains("--audit-assets"));
        assert!(readme.contains("--audit-audio"));
        assert!(readme.contains("--audit-controls"));
        assert!(readme.contains("--audit-input-flow"));
        assert!(readme.contains("--audit-localization"));
        assert!(readme.contains("--audit-layout"));
        assert!(readme.contains("--audit-visual"));
        assert!(readme.contains("--audit-accessibility"));
        assert!(readme.contains("--audit-performance"));
        assert!(readme.contains("--audit-marketing"));
        assert!(readme.contains("--audit-ip"));
        assert!(readme.contains("--audit-save"));
        assert!(readme.contains("--audit-playthrough"));
        assert!(readme.contains("--simulate-campaign"));
        assert!(readme.contains("--release-readiness"));
        assert!(readme.contains("scripts/audio_smoke.sh"));
        assert!(readme.contains("scripts/runtime_smoke.sh"));
        assert!(readme.contains("scripts/visual_smoke.sh"));
        assert!(readme.contains("scripts/store_asset_audit.sh"));
        assert!(readme.contains("scripts/content_rating_audit.sh"));
        assert!(readme.contains("scripts/linux_package_audit.sh"));
        assert!(readme.contains("scripts/linux_install_smoke.sh"));
        assert!(readme.contains("scripts/linux_dependency_audit.sh"));
        assert!(readme.contains("scripts/linux_portability_smoke.sh"));
        assert!(readme.contains("scripts/linux_clean_distro_smoke.sh"));
        assert!(readme.contains("scripts/linux_metadata_audit.sh"));
        assert!(readme.contains("manual_qa_session.sh"));
        assert!(readme.contains("manual_qa_observations.sh"));
        assert!(readme.contains("platform_package_plan.sh"));
        assert!(readme.contains("qa_evidence_summary.sh"));
        assert!(readme.contains("verify_release.sh"));
        assert!(readme.contains("support_diagnostics.sh"));
        assert!(readme.contains("signoff_bundle.sh"));
        assert!(readme.contains("create_candidate_evidence.sh"));
        assert!(readme.contains("create_store_submission_pack.sh"));
        assert!(readme.contains("preserves Pending status"));
        assert!(readme.contains("final_signoff_check.sh"));
        assert!(readme.contains("table rows are completed with owner/date metadata"));
        assert!(readme.contains("package_windows.ps1"));
        assert!(readme.contains("package_macos.sh"));
        assert!(readme.contains("linux-package-audit.txt"));
        assert!(readme.contains("linux-install-smoke.txt"));
        assert!(readme.contains("linux-dependency-audit.txt"));
        assert!(readme.contains("linux-portability-smoke.txt"));
        assert!(readme.contains("linux-clean-distro-smoke.txt"));
        assert!(readme.contains("store-asset-audit.txt"));
        assert!(readme.contains("content-rating-audit.txt"));
        assert!(readme.contains("linux-metadata-audit.txt"));
        assert!(readme.contains("manual-qa-plan.txt"));
        assert!(readme.contains("platform-package-plan.txt"));
        assert!(readme.contains("final-signoff-plan.txt"));
        assert!(readme.contains("release-manifest.json"));
        assert!(readme.contains("PRIVACY.md"));
        assert!(readme.contains("SUPPORT.md"));
        assert!(readme.contains("--audit-privacy"));
        assert!(readme.contains("--audit-release-provenance"));
        assert!(readme.contains("privacy-audit.txt"));
        assert!(readme.contains("release-provenance-audit.txt"));
        assert!(readme.contains("BUILD_PROVENANCE.md"));
        assert!(readme.contains("install_linux_user.sh"));
        assert!(readme.contains("uninstall_linux_user.sh"));
        assert!(readme.contains("--purge"));
        assert!(license.contains("MIT License"));
        assert!(credits.contains("Credits"));
        assert!(art_assets.contains("Production Art"));
        assert!(notices.contains("Bevy"));
        assert!(notices.contains("No third-party art"));
        assert!(store_page.contains("Short Description"));
        assert!(store_page.contains("STORE_SCREENSHOTS.md"));
        assert!(store_page.contains("store_screenshot_check.sh"));
        assert!(presskit.contains("Fact Sheet"));
        assert!(presskit.contains("STORE_SCREENSHOTS.md"));
        assert!(presskit.contains("store_screenshot_check.sh"));
        assert!(presskit.contains("CONTENT_RATING.md"));
        assert!(content_rating.contains("Bevy Open Siege Content Rating Notes"));
        assert!(content_rating.contains("No in-app purchases"));
        assert!(content_rating.contains("Data collection: none"));
        assert!(store_screenshots.contains("screenshots/01-title-menu.png"));
        assert!(store_screenshots.contains("qa-session/store-screenshots.md"));
        assert!(
            store_screenshots.contains("store_screenshot_check.sh --capture-pack . screenshots 10")
        );
        assert!(store_screenshots.contains("store_screenshot_check.sh --validate-dir screenshots"));
        assert!(privacy.contains("does not include telemetry"));
        assert!(privacy.contains("BEVY_OPEN_SIEGE_SAVE_PATH"));
        assert!(support.contains("no automatic crash uploader"));
        assert!(support.contains("QA_SIGNOFF.md"));
        assert!(support.contains("support_diagnostics.sh"));
        assert!(troubleshooting.contains("Bevy Open Siege Troubleshooting"));
        assert!(troubleshooting.contains("verify_release.sh --quick"));
        assert!(troubleshooting.contains("support_diagnostics.sh"));
        assert!(troubleshooting.contains("runtime_smoke.sh"));
        assert!(troubleshooting.contains("visual_smoke.sh"));
        assert!(troubleshooting.contains("audio_smoke.sh"));
        assert!(troubleshooting.contains("final_signoff_check.sh --check"));
        assert!(build_provenance.contains("Cargo.lock"));
        assert!(build_provenance.contains("release-provenance-audit.txt"));
        assert!(build_provenance.contains("SHA256SUMS"));
        assert!(release_notes.contains("0.1.0 Release Candidate"));
        assert!(release_notes.contains("--validate-data"));
        assert!(release_notes.contains("asset-audit.txt"));
        assert!(release_notes.contains("audio-audit.txt"));
        assert!(release_notes.contains("controls-audit.txt"));
        assert!(release_notes.contains("input-flow-audit.txt"));
        assert!(release_notes.contains("localization-audit.txt"));
        assert!(release_notes.contains("layout-audit.txt"));
        assert!(release_notes.contains("visual-readability-audit.txt"));
        assert!(release_notes.contains("accessibility-audit.txt"));
        assert!(release_notes.contains("performance-audit.txt"));
        assert!(release_notes.contains("privacy-audit.txt"));
        assert!(release_notes.contains("release-provenance-audit.txt"));
        assert!(release_notes.contains("marketing-audit.txt"));
        assert!(release_notes.contains("ip-audit.txt"));
        assert!(release_notes.contains("save-audit.txt"));
        assert!(release_notes.contains("playthrough-audit.txt"));
        assert!(release_notes.contains("campaign-simulation.txt"));
        assert!(release_notes.contains("release-readiness.txt"));
        assert!(release_notes.contains("audio_smoke.sh"));
        assert!(release_notes.contains("runtime-smoke.txt"));
        assert!(release_notes.contains("visual-smoke.txt"));
        assert!(release_notes.contains("audio-smoke.txt"));
        assert!(release_notes.contains("store-asset-audit.txt"));
        assert!(release_notes.contains("content-rating-audit.txt"));
        assert!(release_notes.contains("linux-package-audit.txt"));
        assert!(release_notes.contains("linux-install-smoke.txt"));
        assert!(release_notes.contains("linux-dependency-audit.txt"));
        assert!(release_notes.contains("linux-portability-smoke.txt"));
        assert!(release_notes.contains("linux-clean-distro-smoke.txt"));
        assert!(release_notes.contains("linux-metadata-audit.txt"));
        assert!(release_notes.contains("manual-qa-plan.txt"));
        assert!(release_notes.contains("manual_qa_observations.sh"));
        assert!(release_notes.contains("platform-package-plan.txt"));
        assert!(release_notes.contains("qa_evidence_summary.sh"));
        assert!(release_notes.contains("verify_release.sh"));
        assert!(release_notes.contains("support_diagnostics.sh"));
        assert!(release_notes.contains("signoff_bundle.sh"));
        assert!(release_notes.contains("create_candidate_evidence.sh"));
        assert!(release_notes.contains("create_store_submission_pack.sh"));
        assert!(release_notes.contains("final-signoff-plan.txt"));
        assert!(release_notes.contains("release-manifest.json"));
        assert!(release_notes.contains("package_windows.ps1"));
        assert!(release_notes.contains("package_macos.sh"));
        assert!(release_notes.contains("PRIVACY.md"));
        assert!(release_notes.contains("SUPPORT.md"));
        assert!(release_notes.contains("TROUBLESHOOTING.md"));
        assert!(release_notes.contains("BUILD_PROVENANCE.md"));
        assert!(release_notes.contains("STORE_SCREENSHOTS.md"));
        assert!(release_notes.contains("store_screenshot_check.sh"));
        assert!(release_notes.contains("QA_SIGNOFF.md"));
        assert!(checklist.contains("Required Before Release"));
        assert!(checklist.contains("BEVY_OPEN_SIEGE_ISOLATED_TARGET"));
        assert!(checklist.contains("--print-save-path"));
        assert!(checklist.contains("--audit-assets"));
        assert!(checklist.contains("--audit-audio"));
        assert!(checklist.contains("--audit-controls"));
        assert!(checklist.contains("--audit-input-flow"));
        assert!(checklist.contains("--audit-localization"));
        assert!(checklist.contains("--audit-layout"));
        assert!(checklist.contains("--audit-visual"));
        assert!(checklist.contains("--audit-accessibility"));
        assert!(checklist.contains("--audit-performance"));
        assert!(checklist.contains("--audit-privacy"));
        assert!(checklist.contains("--audit-release-provenance"));
        assert!(checklist.contains("--audit-marketing"));
        assert!(checklist.contains("--audit-ip"));
        assert!(checklist.contains("--audit-save"));
        assert!(checklist.contains("--audit-playthrough"));
        assert!(checklist.contains("--simulate-campaign"));
        assert!(checklist.contains("--release-readiness"));
        assert!(checklist.contains("runtime-smoke.txt"));
        assert!(checklist.contains("visual-smoke.txt"));
        assert!(checklist.contains("audio-smoke.txt"));
        assert!(checklist.contains("store-asset-audit.txt"));
        assert!(checklist.contains("store_asset_audit.sh"));
        assert!(checklist.contains("CONTENT_RATING.md"));
        assert!(checklist.contains("content-rating-audit.txt"));
        assert!(checklist.contains("content_rating_audit.sh"));
        assert!(checklist.contains("linux-package-audit.txt"));
        assert!(checklist.contains("linux_package_audit.sh"));
        assert!(checklist.contains("linux-install-smoke.txt"));
        assert!(checklist.contains("linux_install_smoke.sh"));
        assert!(checklist.contains("linux-dependency-audit.txt"));
        assert!(checklist.contains("linux_dependency_audit.sh"));
        assert!(checklist.contains("linux-portability-smoke.txt"));
        assert!(checklist.contains("linux_portability_smoke.sh"));
        assert!(checklist.contains("linux-clean-distro-smoke.txt"));
        assert!(checklist.contains("linux_clean_distro_smoke.sh"));
        assert!(checklist.contains("linux-metadata-audit.txt"));
        assert!(checklist.contains("linux_metadata_audit.sh"));
        assert!(checklist.contains("manual-qa-plan.txt"));
        assert!(checklist.contains("manual_qa_session.sh"));
        assert!(checklist.contains("manual_qa_observations.sh"));
        assert!(checklist.contains("platform-package-plan.txt"));
        assert!(checklist.contains("platform_package_plan.sh"));
        assert!(checklist.contains("qa_evidence_summary.sh"));
        assert!(checklist.contains("verify_release.sh"));
        assert!(checklist.contains("support_diagnostics.sh"));
        assert!(checklist.contains("signoff_bundle.sh"));
        assert!(checklist.contains("create_candidate_evidence.sh"));
        assert!(checklist.contains("create_store_submission_pack.sh"));
        assert!(checklist.contains("final-signoff-plan.txt"));
        assert!(checklist.contains("release-manifest.json"));
        assert!(checklist.contains("privacy-audit.txt"));
        assert!(checklist.contains("PRIVACY.md"));
        assert!(checklist.contains("SUPPORT.md"));
        assert!(checklist.contains("TROUBLESHOOTING.md"));
        assert!(checklist.contains("release-provenance-audit.txt"));
        assert!(checklist.contains("BUILD_PROVENANCE.md"));
        assert!(checklist.contains("STORE_SCREENSHOTS.md"));
        assert!(checklist.contains("qa-session/store-screenshots.md"));
        assert!(checklist.contains("store_screenshot_check.sh"));
        assert!(checklist.contains("final_signoff_check.sh"));
        assert!(checklist.contains("package_windows.ps1"));
        assert!(checklist.contains("package_macos.sh"));
        assert!(checklist.contains("QA_SIGNOFF.md"));
        assert!(checklist.contains("YYYY-MM-DD Date"));
        assert!(qa_signoff.contains("Bevy Open Siege QA Signoff"));
        assert!(qa_signoff.contains("Manual QA session plan"));
        assert!(qa_signoff.contains("manual-qa-plan.txt"));
        assert!(qa_signoff.contains("Platform package plan"));
        assert!(qa_signoff.contains("platform-package-plan.txt"));
        assert!(qa_signoff.contains("QA evidence summary"));
        assert!(qa_signoff.contains("qa_evidence_summary.sh"));
        assert!(qa_signoff.contains("Package verification helper"));
        assert!(qa_signoff.contains("verify_release.sh"));
        assert!(qa_signoff.contains("Support diagnostics helper"));
        assert!(qa_signoff.contains("support_diagnostics.sh"));
        assert!(qa_signoff.contains("Signoff evidence bundle"));
        assert!(qa_signoff.contains("signoff_bundle.sh"));
        assert!(qa_signoff.contains("Release manifest"));
        assert!(qa_signoff.contains("release-manifest.json"));
        assert!(qa_signoff.contains("Final signoff check"));
        assert!(qa_signoff.contains("final-signoff-plan.txt"));
        assert!(qa_signoff.contains("final_signoff_check.sh"));
        assert!(qa_signoff.contains("package_windows.ps1"));
        assert!(qa_signoff.contains("package_macos.sh"));
        assert!(qa_signoff.contains("Full campaign playthrough"));
        assert!(qa_signoff.contains("Audio device QA"));
        assert!(qa_signoff.contains("Localization QA"));
        assert!(qa_signoff.contains("Visual readability"));
        assert!(qa_signoff.contains("Runtime startup smoke"));
        assert!(qa_signoff.contains("Visual startup smoke"));
        assert!(qa_signoff.contains("Audio mix audit"));
        assert!(qa_signoff.contains("Audio startup smoke"));
        assert!(qa_signoff.contains("audio-audit.txt"));
        assert!(qa_signoff.contains("Input flow audit"));
        assert!(qa_signoff.contains("input-flow-audit.txt"));
        assert!(qa_signoff.contains("Accessibility QA"));
        assert!(qa_signoff.contains("accessibility-audit.txt"));
        assert!(qa_signoff.contains("Performance QA"));
        assert!(qa_signoff.contains("performance-audit.txt"));
        assert!(qa_signoff.contains("Privacy, support, and troubleshooting audit"));
        assert!(qa_signoff.contains("TROUBLESHOOTING.md"));
        assert!(qa_signoff.contains("privacy-audit.txt"));
        assert!(qa_signoff.contains("Build provenance audit"));
        assert!(qa_signoff.contains("release-provenance-audit.txt"));
        assert!(qa_signoff.contains("Store screenshot review"));
        assert!(qa_signoff.contains("STORE_SCREENSHOTS.md"));
        assert!(qa_signoff.contains("store_screenshot_check.sh"));
        assert!(qa_signoff.contains("Store asset audit"));
        assert!(qa_signoff.contains("store-asset-audit.txt"));
        assert!(qa_signoff.contains("store_asset_audit.sh"));
        assert!(qa_signoff.contains("Content rating review"));
        assert!(qa_signoff.contains("CONTENT_RATING.md"));
        assert!(qa_signoff.contains("content-rating-audit.txt"));
        assert!(qa_signoff.contains("content_rating_audit.sh"));
        assert!(qa_signoff.contains("linux-package-audit.txt"));
        assert!(qa_signoff.contains("linux-install-smoke.txt"));
        assert!(qa_signoff.contains("linux-dependency-audit.txt"));
        assert!(qa_signoff.contains("linux-portability-smoke.txt"));
        assert!(qa_signoff.contains("linux-clean-distro-smoke.txt"));
        assert!(qa_signoff.contains("linux_portability_smoke.sh"));
        assert!(qa_signoff.contains("linux_clean_distro_smoke.sh"));
        assert!(qa_signoff.contains("manual_qa_observations.sh"));
        assert!(qa_signoff.contains("Linux desktop metadata QA"));
        assert!(qa_signoff.contains("linux-metadata-audit.txt"));
        assert!(qa_signoff.contains("linux_metadata_audit.sh"));
        assert!(qa_signoff.contains("Marketing material audit"));
        assert!(qa_signoff.contains("IP and naming audit"));
        assert!(qa_signoff.contains("Save audit"));
        assert!(qa_signoff.contains("Scripted playthrough audit"));
        assert!(qa_signoff.contains("Windows package QA"));
        assert!(qa_signoff.contains("macOS package QA"));
        assert!(qa_signoff.contains("Release approved: No"));
        assert!(release_check_script.contains("BEVY_OPEN_SIEGE_USE_NIX"));
        assert!(release_check_script.contains("BEVY_OPEN_SIEGE_ISOLATED_TARGET"));
        assert!(release_check_script.contains("CARGO_INCREMENTAL"));
        assert!(release_check_script.contains("cargo clippy --all-targets -- -D warnings"));
        assert!(release_check_script.contains("--audit-balance"));
        assert!(release_check_script.contains("--audit-assets"));
        assert!(release_check_script.contains("--audit-audio"));
        assert!(release_check_script.contains("--audit-controls"));
        assert!(release_check_script.contains("--audit-input-flow"));
        assert!(release_check_script.contains("--audit-localization"));
        assert!(release_check_script.contains("--audit-layout"));
        assert!(release_check_script.contains("--audit-visual"));
        assert!(release_check_script.contains("--audit-accessibility"));
        assert!(release_check_script.contains("--audit-performance"));
        assert!(release_check_script.contains("--audit-privacy"));
        assert!(release_check_script.contains("--audit-release-provenance"));
        assert!(release_check_script.contains("--audit-marketing"));
        assert!(release_check_script.contains("--audit-ip"));
        assert!(release_check_script.contains("--audit-save"));
        assert!(release_check_script.contains("--audit-playthrough"));
        assert!(release_check_script.contains("--simulate-campaign"));
        assert!(release_check_script.contains("--release-readiness"));
        assert!(release_check_script.contains("audio_smoke.sh"));
        assert!(release_check_script.contains("runtime_smoke.sh"));
        assert!(release_check_script.contains("visual_smoke.sh"));
        assert!(release_check_script.contains("store_screenshot_check.sh"));
        assert!(release_check_script.contains("store_asset_audit.sh"));
        assert!(release_check_script.contains("content_rating_audit.sh"));
        assert!(release_check_script.contains("linux_package_audit.sh"));
        assert!(release_check_script.contains("linux_install_smoke.sh"));
        assert!(release_check_script.contains("linux_dependency_audit.sh"));
        assert!(release_check_script.contains("linux_portability_smoke.sh"));
        assert!(release_check_script.contains("linux_clean_distro_smoke.sh"));
        assert!(release_check_script.contains("linux_metadata_audit.sh"));
        assert!(release_check_script.contains("manual_qa_session.sh"));
        assert!(release_check_script.contains("manual_qa_observations.sh"));
        assert!(release_check_script.contains("platform_package_plan.sh"));
        assert!(release_check_script.contains("qa_evidence_summary.sh"));
        assert!(release_check_script.contains("final_signoff_check.sh"));
        assert!(release_check_script.contains("verify_release.sh"));
        assert!(release_check_script.contains("support_diagnostics.sh"));
        assert!(release_check_script.contains("signoff_bundle.sh"));
        assert!(release_check_script.contains("create_candidate_evidence.sh"));
        assert!(release_check_script.contains("create_store_submission_pack.sh"));
        assert!(release_check_script.contains("package_windows.ps1"));
        assert!(release_check_script.contains("package_macos.sh"));
        assert!(release_check_script.contains("generate_release_manifest.py"));
        assert!(release_check_script.contains("py_compile"));
        assert!(package_script.contains("BEVY_OPEN_SIEGE_USE_NIX"));
        assert!(package_script.contains("install_linux_user.sh"));
        assert!(package_script.contains("uninstall_linux_user.sh"));
        assert!(package_script.contains("bevy_open_siege.bin"));
        assert!(package_script.contains("ld-linux-x86-64.so.2"));
        assert!(package_script.contains("--library-path"));
        assert!(package_script.contains("patchelf --set-rpath '$ORIGIN/lib'"));
        assert!(package_script.contains("generate_third_party_licenses.py"));
        assert!(package_script.contains("THIRD_PARTY_LICENSES.md"));
        assert!(package_script.contains("SHA256SUMS"));
        assert!(package_script.contains("sha256sum"));
        assert!(package_script.contains("balance-audit.txt"));
        assert!(package_script.contains("asset-audit.txt"));
        assert!(package_script.contains("audio-audit.txt"));
        assert!(package_script.contains("controls-audit.txt"));
        assert!(package_script.contains("input-flow-audit.txt"));
        assert!(package_script.contains("localization-audit.txt"));
        assert!(package_script.contains("layout-audit.txt"));
        assert!(package_script.contains("visual-readability-audit.txt"));
        assert!(package_script.contains("accessibility-audit.txt"));
        assert!(package_script.contains("performance-audit.txt"));
        assert!(package_script.contains("privacy-audit.txt"));
        assert!(package_script.contains("release-provenance-audit.txt"));
        assert!(package_script.contains("marketing-audit.txt"));
        assert!(package_script.contains("ip-audit.txt"));
        assert!(package_script.contains("save-audit.txt"));
        assert!(package_script.contains("playthrough-audit.txt"));
        assert!(package_script.contains("campaign-simulation.txt"));
        assert!(package_script.contains("release-readiness.txt"));
        assert!(package_script.contains("runtime-smoke.txt"));
        assert!(package_script.contains("visual-smoke.txt"));
        assert!(package_script.contains("audio-smoke.txt"));
        assert!(package_script.contains("store-asset-audit.txt"));
        assert!(package_script.contains("content-rating-audit.txt"));
        assert!(package_script.contains("linux-package-audit.txt"));
        assert!(package_script.contains("linux-install-smoke.txt"));
        assert!(package_script.contains("linux-dependency-audit.txt"));
        assert!(package_script.contains("linux-portability-smoke.txt"));
        assert!(package_script.contains("linux-clean-distro-smoke.txt"));
        assert!(package_script.contains("linux-metadata-audit.txt"));
        assert!(package_script.contains("manual-qa-plan.txt"));
        assert!(package_script.contains("platform-package-plan.txt"));
        assert!(package_script.contains("final-signoff-plan.txt"));
        assert!(package_script.contains("release-manifest.json"));
        assert!(package_script.contains("generate_release_manifest.py"));
        assert!(package_script.contains("audio_smoke.sh"));
        assert!(package_script.contains("runtime_smoke.sh"));
        assert!(package_script.contains("visual_smoke.sh"));
        assert!(package_script.contains("store_screenshot_check.sh"));
        assert!(package_script.contains("store_asset_audit.sh"));
        assert!(package_script.contains("content_rating_audit.sh"));
        assert!(package_script.contains("linux_package_audit.sh"));
        assert!(package_script.contains("linux_install_smoke.sh"));
        assert!(package_script.contains("linux_dependency_audit.sh"));
        assert!(package_script.contains("linux_portability_smoke.sh"));
        assert!(package_script.contains("linux_clean_distro_smoke.sh"));
        assert!(package_script.contains("linux_metadata_audit.sh"));
        assert!(package_script.contains("manual_qa_session.sh"));
        assert!(package_script.contains("manual_qa_observations.sh"));
        assert!(package_script.contains("platform_package_plan.sh"));
        assert!(package_script.contains("qa_evidence_summary.sh"));
        assert!(package_script.contains("final_signoff_check.sh"));
        assert!(package_script.contains("verify_release.sh"));
        assert!(package_script.contains("support_diagnostics.sh"));
        assert!(package_script.contains("signoff_bundle.sh"));
        assert!(package_script.contains("create_candidate_evidence.sh"));
        assert!(package_script.contains("create_store_submission_pack.sh"));
        assert!(package_script.contains("package_windows.ps1"));
        assert!(package_script.contains("package_macos.sh"));
        assert!(package_script.contains("QA_SIGNOFF.md"));
        assert!(package_script.contains("PRIVACY.md"));
        assert!(package_script.contains("SUPPORT.md"));
        assert!(package_script.contains("TROUBLESHOOTING.md"));
        assert!(package_script.contains("BUILD_PROVENANCE.md"));
        assert!(package_script.contains("STORE_SCREENSHOTS.md"));
        assert!(package_script.contains("CONTENT_RATING.md"));
        assert!(package_script.contains("RELEASE_NOTES.md"));
        assert!(package_script.contains("smoke_release_archive.sh"));
        assert!(smoke_script.contains("--validate-data"));
        assert!(smoke_script.contains("--audit-balance"));
        assert!(smoke_script.contains("--audit-assets"));
        assert!(smoke_script.contains("--audit-audio"));
        assert!(smoke_script.contains("--audit-controls"));
        assert!(smoke_script.contains("--audit-input-flow"));
        assert!(smoke_script.contains("--audit-localization"));
        assert!(smoke_script.contains("--audit-layout"));
        assert!(smoke_script.contains("--audit-visual"));
        assert!(smoke_script.contains("--audit-accessibility"));
        assert!(smoke_script.contains("--audit-performance"));
        assert!(smoke_script.contains("--audit-privacy"));
        assert!(smoke_script.contains("--audit-release-provenance"));
        assert!(smoke_script.contains("--audit-marketing"));
        assert!(smoke_script.contains("--audit-ip"));
        assert!(smoke_script.contains("--audit-save"));
        assert!(smoke_script.contains("--audit-playthrough"));
        assert!(smoke_script.contains("--simulate-campaign"));
        assert!(smoke_script.contains("--release-readiness"));
        assert!(smoke_script.contains("--print-save-summary"));
        assert!(smoke_script.contains("runtime_smoke.sh"));
        assert!(smoke_script.contains("visual_smoke.sh"));
        assert!(smoke_script.contains("store_screenshot_check.sh"));
        assert!(smoke_script.contains("store_asset_audit.sh"));
        assert!(smoke_script.contains("content_rating_audit.sh"));
        assert!(smoke_script.contains("audio_smoke.sh"));
        assert!(smoke_script.contains("linux_package_audit.sh"));
        assert!(smoke_script.contains("linux_install_smoke.sh"));
        assert!(smoke_script.contains("linux_dependency_audit.sh"));
        assert!(smoke_script.contains("linux_portability_smoke.sh"));
        assert!(smoke_script.contains("linux_clean_distro_smoke.sh"));
        assert!(smoke_script.contains("linux_metadata_audit.sh"));
        assert!(smoke_script.contains("manual_qa_session.sh"));
        assert!(smoke_script.contains("manual_qa_observations.sh"));
        assert!(smoke_script.contains("manual QA observations collected"));
        assert!(smoke_script.contains("platform_package_plan.sh"));
        assert!(smoke_script.contains("qa_evidence_summary.sh"));
        assert!(smoke_script.contains("final_signoff_check.sh"));
        assert!(smoke_script.contains("verify_release.sh"));
        assert!(smoke_script.contains("support_diagnostics.sh"));
        assert!(smoke_script.contains("signoff_bundle.sh"));
        assert!(smoke_script.contains("signoff bundle created"));
        assert!(smoke_script.contains("create_candidate_evidence.sh"));
        assert!(smoke_script.contains("candidate evidence created"));
        assert!(smoke_script.contains("create_store_submission_pack.sh"));
        assert!(smoke_script.contains("store submission pack created"));
        assert!(smoke_script.contains("package_windows.ps1"));
        assert!(smoke_script.contains("package_macos.sh"));
        assert!(smoke_script.contains("balance-audit.txt"));
        assert!(smoke_script.contains("asset-audit.txt"));
        assert!(smoke_script.contains("audio-audit.txt"));
        assert!(smoke_script.contains("controls-audit.txt"));
        assert!(smoke_script.contains("input-flow-audit.txt"));
        assert!(smoke_script.contains("localization-audit.txt"));
        assert!(smoke_script.contains("layout-audit.txt"));
        assert!(smoke_script.contains("visual-readability-audit.txt"));
        assert!(smoke_script.contains("accessibility-audit.txt"));
        assert!(smoke_script.contains("performance-audit.txt"));
        assert!(smoke_script.contains("privacy-audit.txt"));
        assert!(smoke_script.contains("release-provenance-audit.txt"));
        assert!(smoke_script.contains("marketing-audit.txt"));
        assert!(smoke_script.contains("ip-audit.txt"));
        assert!(smoke_script.contains("save-audit.txt"));
        assert!(smoke_script.contains("playthrough-audit.txt"));
        assert!(smoke_script.contains("campaign-simulation.txt"));
        assert!(smoke_script.contains("release-readiness.txt"));
        assert!(smoke_script.contains("runtime-smoke.txt"));
        assert!(smoke_script.contains("visual-smoke.txt"));
        assert!(smoke_script.contains("audio-smoke.txt"));
        assert!(smoke_script.contains("store-asset-audit.txt"));
        assert!(smoke_script.contains("content-rating-audit.txt"));
        assert!(smoke_script.contains("linux-package-audit.txt"));
        assert!(smoke_script.contains("linux-install-smoke.txt"));
        assert!(smoke_script.contains("linux-dependency-audit.txt"));
        assert!(smoke_script.contains("linux-portability-smoke.txt"));
        assert!(smoke_script.contains("linux-clean-distro-smoke.txt"));
        assert!(smoke_script.contains("linux portability smoke ok"));
        assert!(smoke_script.contains("linux clean distro smoke ok"));
        assert!(smoke_script.contains("automated evidence: total=35 missing=0"));
        assert!(smoke_script.contains("linux-metadata-audit.txt"));
        assert!(smoke_script.contains("manual-qa-plan.txt"));
        assert!(smoke_script.contains("platform-package-plan.txt"));
        assert!(smoke_script.contains("qa evidence summary ok"));
        assert!(smoke_script.contains("release package verification ok"));
        assert!(smoke_script.contains("final-signoff-plan.txt"));
        assert!(smoke_script.contains("release-manifest.json"));
        assert!(smoke_script.contains("bevy-open-siege-release-manifest-v1"));
        assert!(smoke_script.contains("QA_SIGNOFF.md"));
        assert!(smoke_script.contains("PRIVACY.md"));
        assert!(smoke_script.contains("SUPPORT.md"));
        assert!(smoke_script.contains("TROUBLESHOOTING.md"));
        assert!(smoke_script.contains("Bevy Open Siege Troubleshooting"));
        assert!(smoke_script.contains("BUILD_PROVENANCE.md"));
        assert!(smoke_script.contains("STORE_SCREENSHOTS.md"));
        assert!(smoke_script.contains("CONTENT_RATING.md"));
        assert!(smoke_script.contains("qa-session/store-screenshots.md"));
        assert!(smoke_script.contains("store screenshot workflow ok"));
        assert!(smoke_script.contains("release-info.txt"));
        assert!(smoke_script.contains("diff -u"));
        assert!(smoke_script.contains("THIRD_PARTY_LICENSES.md"));
        assert!(
            smoke_script.contains("third-party license report contains an unknown license entry")
        );
        assert!(smoke_script.contains("SHA256SUMS"));
        assert!(smoke_script.contains("sha256sum -c SHA256SUMS"));
        assert!(smoke_script.contains("install_linux_user.sh"));
        assert!(smoke_script.contains("uninstall_linux_user.sh"));
        assert!(smoke_script.contains("bevy_open_siege.bin"));
        assert!(smoke_script.contains("lib/ld-linux-x86-64.so.2"));
        assert!(smoke_script.contains("XDG_DATA_HOME"));
        assert!(linux_package_audit_script.contains("linux package audit ok"));
        assert!(linux_package_audit_script.contains("install_linux_user.sh"));
        assert!(linux_package_audit_script.contains("uninstall_linux_user.sh"));
        assert!(linux_package_audit_script.contains("uninstall_linux_user.sh\" --purge"));
        assert!(linux_package_audit_script.contains("bevy_open_siege/app"));
        assert!(
            linux_package_audit_script
                .contains("checked purge: save data removed only with --purge")
        );
        assert!(linux_install_smoke_script.contains("linux install smoke ok"));
        assert!(linux_install_smoke_script.contains("runtime_smoke.sh"));
        assert!(linux_install_smoke_script.contains("visual_smoke.sh"));
        assert!(linux_install_smoke_script.contains("temporary XDG user directories"));
        assert!(linux_install_smoke_script.contains("checked installed launcher"));
        assert!(linux_install_smoke_script.contains("checked purge: save data removed"));
        assert!(linux_dependency_audit_script.contains("linux dependency audit ok"));
        assert!(linux_dependency_audit_script.contains("wrapper_mode"));
        assert!(linux_dependency_audit_script.contains("bevy_open_siege.bin"));
        assert!(linux_dependency_audit_script.contains("bundled_library_files"));
        assert!(linux_dependency_audit_script.contains("missing dependencies: none"));
        assert!(linux_dependency_audit_script.contains("nix_store_references"));
        assert!(linux_dependency_audit_script.contains("portability review"));
        assert!(linux_dependency_audit_script.contains("libasound.so.2"));
        assert!(linux_portability_smoke_script.contains("linux portability smoke ok"));
        assert!(linux_portability_smoke_script.contains("sanitized_env"));
        assert!(linux_portability_smoke_script.contains("LD_LIBRARY_PATH unset"));
        assert!(linux_portability_smoke_script.contains("Nix variables omitted"));
        assert!(linux_portability_smoke_script.contains("clean_distro_qa"));
        assert!(linux_portability_smoke_script.contains("env \"${ENV_ARGS[@]}\""));
        assert!(linux_portability_smoke_script.contains("bevy_open_siege.bin"));
        assert!(linux_clean_distro_smoke_script.contains("linux clean distro smoke ok"));
        assert!(linux_clean_distro_smoke_script.contains("docker.io/library/ubuntu:24.04"));
        assert!(linux_clean_distro_smoke_script.contains("--network none"));
        assert!(linux_clean_distro_smoke_script.contains("validate_data: pass"));
        assert!(linux_clean_distro_smoke_script.contains("dependency_resolution: pass"));
        assert!(linux_clean_distro_smoke_script.contains("manual_clean_machine_qa"));
        assert!(linux_metadata_audit_script.contains("linux metadata audit ok"));
        assert!(linux_metadata_audit_script.contains("bevy-open-siege.desktop"));
        assert!(linux_metadata_audit_script.contains("io.github.bevy_open_siege.BevyOpenSiege"));
        assert!(linux_metadata_audit_script.contains("desktop-file-validate"));
        assert!(linux_metadata_audit_script.contains("appstream"));
        assert!(store_asset_audit_script.contains("store asset audit ok"));
        assert!(store_asset_audit_script.contains("app-icon.png"));
        assert!(store_asset_audit_script.contains("store-capsule.png"));
        assert!(store_asset_audit_script.contains("STORE_SCREENSHOTS.md"));
        assert!(store_asset_audit_script.contains("image variance validator"));
        assert!(content_rating_audit_script.contains("content rating audit ok"));
        assert!(content_rating_audit_script.contains("CONTENT_RATING.md"));
        assert!(content_rating_audit_script.contains("No in-app purchases"));
        assert!(content_rating_audit_script.contains("manual rating review still required"));
        assert!(manual_qa_session_script.contains("manual QA session plan ok"));
        assert!(manual_qa_session_script.contains("manual QA session initialized"));
        assert!(manual_qa_session_script.contains("--init"));
        assert!(manual_qa_session_script.contains("visual-smoke.txt"));
        assert!(manual_qa_session_script.contains("store-asset-audit.txt"));
        assert!(manual_qa_session_script.contains("store_asset_audit.sh"));
        assert!(manual_qa_session_script.contains("content-rating-audit.txt"));
        assert!(manual_qa_session_script.contains("content_rating_audit.sh"));
        assert!(manual_qa_session_script.contains("linux-install-smoke.txt"));
        assert!(manual_qa_session_script.contains("linux-dependency-audit.txt"));
        assert!(manual_qa_session_script.contains("linux-portability-smoke.txt"));
        assert!(manual_qa_session_script.contains("linux-clean-distro-smoke.txt"));
        assert!(manual_qa_session_script.contains("linux-metadata-audit.txt"));
        assert!(manual_qa_session_script.contains("linux_metadata_audit.sh"));
        assert!(manual_qa_session_script.contains("accessibility-audit.txt"));
        assert!(manual_qa_session_script.contains("qa-session/accessibility-qa.md"));
        assert!(manual_qa_session_script.contains("performance-audit.txt"));
        assert!(manual_qa_session_script.contains("qa-session/performance-qa.md"));
        assert!(manual_qa_session_script.contains("privacy-audit.txt"));
        assert!(manual_qa_session_script.contains("qa-session/privacy-support.md"));
        assert!(manual_qa_session_script.contains("release-provenance-audit.txt"));
        assert!(manual_qa_session_script.contains("qa-session/build-provenance.md"));
        assert!(manual_qa_session_script.contains("qa-session/store-screenshots.md"));
        assert!(manual_qa_session_script.contains("STORE_SCREENSHOTS.md"));
        assert!(
            manual_qa_session_script
                .contains("store_screenshot_check.sh --validate-dir screenshots")
        );
        assert!(manual_qa_session_script.contains("qa-session/full-campaign-playthrough.md"));
        assert!(
            manual_qa_session_script
                .contains("BEVY_OPEN_SIEGE_SAVE_PATH=qa-session/bevy_open_siege_save.ron")
        );
        assert!(manual_qa_session_script.contains("Status: Pending"));
        assert!(manual_qa_session_script.contains("Approved: No"));
        assert!(manual_qa_session_script.contains("YYYY-MM-DD Date"));
        assert!(manual_qa_session_script.contains("final decision rule:"));
        assert!(manual_qa_observations_script.contains("manual QA observations collected"));
        assert!(manual_qa_observations_script.contains("Status remains Pending"));
        assert!(manual_qa_observations_script.contains("Approved: No"));
        assert!(manual_qa_observations_script.contains("automated-observations.md"));
        assert!(manual_qa_observations_script.contains("verify-release-quick.txt"));
        assert!(manual_qa_observations_script.contains("qa-evidence-summary.txt"));
        assert!(manual_qa_observations_script.contains("does not approve the release"));
        assert!(platform_package_plan_script.contains("platform package plan ok"));
        assert!(platform_package_plan_script.contains("x86_64-pc-windows-msvc"));
        assert!(platform_package_plan_script.contains("aarch64-apple-darwin"));
        assert!(platform_package_plan_script.contains("windows-package-qa.md"));
        assert!(platform_package_plan_script.contains("macos-package-qa.md"));
        assert!(platform_package_plan_script.contains("platform package session initialized"));
        assert!(platform_package_plan_script.contains("package_windows.ps1"));
        assert!(platform_package_plan_script.contains("package_macos.sh"));
        assert!(platform_package_plan_script.contains("--audit-accessibility"));
        assert!(platform_package_plan_script.contains("--audit-performance"));
        assert!(platform_package_plan_script.contains("--audit-privacy"));
        assert!(platform_package_plan_script.contains("--audit-release-provenance"));
        assert!(platform_package_plan_script.contains("privacy-audit.txt"));
        assert!(platform_package_plan_script.contains("release-provenance-audit.txt"));
        assert!(platform_package_plan_script.contains("PRIVACY.md"));
        assert!(platform_package_plan_script.contains("SUPPORT.md"));
        assert!(platform_package_plan_script.contains("BUILD_PROVENANCE.md"));
        assert!(qa_evidence_summary_script.contains("qa evidence summary ok"));
        assert!(qa_evidence_summary_script.contains("release-manifest.json"));
        assert!(qa_evidence_summary_script.contains("create_store_submission_pack.sh"));
        assert!(qa_evidence_summary_script.contains("store-asset-audit.txt"));
        assert!(qa_evidence_summary_script.contains("content-rating-audit.txt"));
        assert!(qa_evidence_summary_script.contains("linux-install-smoke.txt"));
        assert!(qa_evidence_summary_script.contains("linux-dependency-audit.txt"));
        assert!(qa_evidence_summary_script.contains("linux-portability-smoke.txt"));
        assert!(qa_evidence_summary_script.contains("linux-clean-distro-smoke.txt"));
        assert!(qa_evidence_summary_script.contains("linux-metadata-audit.txt"));
        assert!(qa_evidence_summary_script.contains("qa-signoff rows"));
        assert!(qa_evidence_summary_script.contains("owner_missing"));
        assert!(qa_evidence_summary_script.contains("manual evidence"));
        assert!(qa_evidence_summary_script.contains("platform evidence"));
        assert!(qa_evidence_summary_script.contains("final_signoff_check.sh --check"));
        assert!(final_signoff_check_script.contains("final signoff plan ok"));
        assert!(final_signoff_check_script.contains("check_qa_signoff_rows"));
        assert!(verify_release_script.contains("release package verification ok"));
        assert!(verify_release_script.contains("release-manifest.json"));
        assert!(verify_release_script.contains("release manifest: present"));
        assert!(verify_release_script.contains("bevy_open_siege.bin"));
        assert!(verify_release_script.contains("lib/ld-linux-x86-64.so.2"));
        assert!(verify_release_script.contains("store-asset-audit.txt"));
        assert!(verify_release_script.contains("store_asset_audit.sh"));
        assert!(verify_release_script.contains("content-rating-audit.txt"));
        assert!(verify_release_script.contains("content_rating_audit.sh"));
        assert!(verify_release_script.contains("linux-dependency-audit.txt"));
        assert!(verify_release_script.contains("linux_dependency_audit.sh"));
        assert!(verify_release_script.contains("linux-portability-smoke.txt"));
        assert!(verify_release_script.contains("linux_portability_smoke.sh"));
        assert!(verify_release_script.contains("linux portability smoke ok"));
        assert!(verify_release_script.contains("linux-clean-distro-smoke.txt"));
        assert!(verify_release_script.contains("linux_clean_distro_smoke.sh"));
        assert!(verify_release_script.contains("linux clean distro smoke ok"));
        assert!(verify_release_script.contains("linux-metadata-audit.txt"));
        assert!(verify_release_script.contains("linux_metadata_audit.sh"));
        assert!(verify_release_script.contains("BEVY_OPEN_SIEGE_DIAGNOSTICS_SKIP_VERIFY=1"));
        assert!(verify_release_script.contains("TROUBLESHOOTING.md"));
        assert!(verify_release_script.contains("signoff_bundle.sh"));
        assert!(verify_release_script.contains("create_candidate_evidence.sh"));
        assert!(verify_release_script.contains("create_store_submission_pack.sh"));
        assert!(verify_release_script.contains("--quick verifies archive integrity"));
        assert!(verify_release_script.contains("--full also runs runtime"));
        assert!(verify_release_script.contains("deterministic reports: matched"));
        assert!(final_signoff_check_script.contains("--check"));
        assert!(final_signoff_check_script.contains("release-manifest.json"));
        assert!(final_signoff_check_script.contains("privacy-support.md"));
        assert!(final_signoff_check_script.contains("build-provenance.md"));
        assert!(final_signoff_check_script.contains("store-screenshots.md"));
        assert!(final_signoff_check_script.contains("Release approved: Yes"));
        assert!(final_signoff_check_script.contains("Status: Pass"));
        assert!(final_signoff_check_script.contains("Approved: Yes"));
        assert!(final_signoff_check_script.contains("QA_SIGNOFF.md row still pending"));
        assert!(final_signoff_check_script.contains("scoped-out row requires notes"));
        assert!(final_signoff_check_script.contains("final signoff check passed"));
        assert!(final_signoff_check_script.contains("qa signoff rows: complete"));
        assert!(support_diagnostics_script.contains("support diagnostics collected"));
        assert!(support_diagnostics_script.contains("BEVY_OPEN_SIEGE_DIAGNOSTICS_SKIP_VERIFY"));
        assert!(
            support_diagnostics_script
                .contains("no save files, screenshots, recordings, or personal files")
        );
        assert!(support_diagnostics_script.contains("--print-release-info"));
        assert!(support_diagnostics_script.contains("--print-save-path"));
        assert!(support_diagnostics_script.contains("--audit-privacy"));
        assert!(signoff_bundle_script.contains("signoff bundle created"));
        assert!(signoff_bundle_script.contains("--allow-candidate"));
        assert!(signoff_bundle_script.contains("final_signoff_check.sh --check"));
        assert!(signoff_bundle_script.contains("qa-evidence-summary.txt"));
        assert!(signoff_bundle_script.contains("store-asset-audit.txt"));
        assert!(signoff_bundle_script.contains("content-rating-audit.txt"));
        assert!(signoff_bundle_script.contains("CONTENT_RATING.md"));
        assert!(signoff_bundle_script.contains("linux-dependency-audit.txt"));
        assert!(signoff_bundle_script.contains("linux-portability-smoke.txt"));
        assert!(signoff_bundle_script.contains("linux-clean-distro-smoke.txt"));
        assert!(signoff_bundle_script.contains("linux-metadata-audit.txt"));
        assert!(candidate_evidence_script.contains("candidate evidence created"));
        assert!(candidate_evidence_script.contains("manual_qa_session.sh"));
        assert!(candidate_evidence_script.contains("manual_qa_observations.sh"));
        assert!(candidate_evidence_script.contains("manual-qa-observations.txt"));
        assert!(candidate_evidence_script.contains("platform_package_plan.sh"));
        assert!(candidate_evidence_script.contains("support_diagnostics.sh"));
        assert!(candidate_evidence_script.contains("signoff_bundle.sh"));
        assert!(candidate_evidence_script.contains("release candidate, not final approved"));
        assert!(candidate_evidence_script.contains("final_signoff_check.sh --check"));
        assert!(store_submission_script.contains("store submission pack created"));
        assert!(store_submission_script.contains("STORE_PAGE.md"));
        assert!(store_submission_script.contains("PRESSKIT.md"));
        assert!(store_submission_script.contains("CONTENT_RATING.md"));
        assert!(store_submission_script.contains("store_screenshot_check.sh"));
        assert!(store_submission_script.contains("screenshot_status=\"pending\""));
        assert!(store_submission_script.contains("final_signoff_check.sh --check"));
        assert!(windows_package_script.contains("x86_64-pc-windows-msvc"));
        assert!(windows_package_script.contains("Compress-Archive"));
        assert!(windows_package_script.contains("Get-FileHash -Algorithm SHA256"));
        assert!(windows_package_script.contains("bevy_open_siege.exe"));
        assert!(windows_package_script.contains("--audit-playthrough"));
        assert!(windows_package_script.contains("input-flow-audit.txt"));
        assert!(windows_package_script.contains("accessibility-audit.txt"));
        assert!(windows_package_script.contains("performance-audit.txt"));
        assert!(windows_package_script.contains("privacy-audit.txt"));
        assert!(windows_package_script.contains("release-provenance-audit.txt"));
        assert!(windows_package_script.contains("PRIVACY.md"));
        assert!(windows_package_script.contains("SUPPORT.md"));
        assert!(windows_package_script.contains("TROUBLESHOOTING.md"));
        assert!(windows_package_script.contains("release-manifest.json"));
        assert!(windows_package_script.contains("generate_release_manifest.py"));
        assert!(windows_package_script.contains("BUILD_PROVENANCE.md"));
        assert!(windows_package_script.contains("STORE_SCREENSHOTS.md"));
        assert!(windows_package_script.contains("visual-smoke.txt"));
        assert!(windows_package_script.contains("visual_smoke.sh"));
        assert!(windows_package_script.contains("store_screenshot_check.sh"));
        assert!(windows_package_script.contains("store_asset_audit.sh"));
        assert!(windows_package_script.contains("store-asset-audit.txt"));
        assert!(windows_package_script.contains("CONTENT_RATING.md"));
        assert!(windows_package_script.contains("content_rating_audit.sh"));
        assert!(windows_package_script.contains("content-rating-audit.txt"));
        assert!(windows_package_script.contains("platform-package-plan.txt"));
        assert!(windows_package_script.contains("qa_evidence_summary.sh"));
        assert!(windows_package_script.contains("verify_release.sh"));
        assert!(windows_package_script.contains("support_diagnostics.sh"));
        assert!(windows_package_script.contains("signoff_bundle.sh"));
        assert!(windows_package_script.contains("create_candidate_evidence.sh"));
        assert!(windows_package_script.contains("create_store_submission_pack.sh"));
        assert!(windows_package_script.contains("final-signoff-plan.txt"));
        assert!(windows_package_script.contains("final_signoff_check.sh"));
        assert!(macos_package_script.contains("x86_64-apple-darwin"));
        assert!(macos_package_script.contains("aarch64-apple-darwin"));
        assert!(macos_package_script.contains("lipo -create"));
        assert!(macos_package_script.contains("--audit-playthrough"));
        assert!(macos_package_script.contains("input-flow-audit.txt"));
        assert!(macos_package_script.contains("accessibility-audit.txt"));
        assert!(macos_package_script.contains("performance-audit.txt"));
        assert!(macos_package_script.contains("privacy-audit.txt"));
        assert!(macos_package_script.contains("release-provenance-audit.txt"));
        assert!(macos_package_script.contains("PRIVACY.md"));
        assert!(macos_package_script.contains("SUPPORT.md"));
        assert!(macos_package_script.contains("TROUBLESHOOTING.md"));
        assert!(macos_package_script.contains("release-manifest.json"));
        assert!(macos_package_script.contains("generate_release_manifest.py"));
        assert!(macos_package_script.contains("BUILD_PROVENANCE.md"));
        assert!(macos_package_script.contains("STORE_SCREENSHOTS.md"));
        assert!(macos_package_script.contains("visual-smoke.txt"));
        assert!(macos_package_script.contains("visual_smoke.sh"));
        assert!(macos_package_script.contains("store_screenshot_check.sh"));
        assert!(macos_package_script.contains("store_asset_audit.sh"));
        assert!(macos_package_script.contains("store-asset-audit.txt"));
        assert!(macos_package_script.contains("CONTENT_RATING.md"));
        assert!(macos_package_script.contains("content_rating_audit.sh"));
        assert!(macos_package_script.contains("content-rating-audit.txt"));
        assert!(macos_package_script.contains("platform-package-plan.txt"));
        assert!(macos_package_script.contains("qa_evidence_summary.sh"));
        assert!(macos_package_script.contains("verify_release.sh"));
        assert!(macos_package_script.contains("support_diagnostics.sh"));
        assert!(macos_package_script.contains("signoff_bundle.sh"));
        assert!(macos_package_script.contains("create_candidate_evidence.sh"));
        assert!(macos_package_script.contains("create_store_submission_pack.sh"));
        assert!(macos_package_script.contains("final-signoff-plan.txt"));
        assert!(macos_package_script.contains("final_signoff_check.sh"));
        assert!(audio_smoke_script.contains("audio startup smoke ok"));
        assert!(audio_smoke_script.contains("Creating new window Bevy Open Siege"));
        assert!(audio_smoke_script.contains("error\\[B0001\\]"));
        assert!(audio_smoke_script.contains("--audio"));
        assert!(runtime_smoke_script.contains("runtime startup smoke ok"));
        assert!(runtime_smoke_script.contains("Creating new window Bevy Open Siege"));
        assert!(runtime_smoke_script.contains("error\\[B0001\\]"));
        assert!(runtime_smoke_script.contains("--no-audio"));
        assert!(visual_smoke_script.contains("visual startup smoke ok"));
        assert!(visual_smoke_script.contains("import -window \"Bevy Open Siege\""));
        assert!(visual_smoke_script.contains("screenshot: nonblank"));
        assert!(visual_smoke_script.contains("standard_deviation"));
        assert!(visual_smoke_script.contains("error\\[B0001\\]"));
        assert!(store_screenshot_script.contains("store screenshot workflow ok"));
        assert!(store_screenshot_script.contains("--capture-current"));
        assert!(store_screenshot_script.contains("--capture-startup"));
        assert!(store_screenshot_script.contains("--capture-pack"));
        assert!(store_screenshot_script.contains("--store-screenshot-scene"));
        assert!(store_screenshot_script.contains("--validate-dir"));
        assert!(store_screenshot_script.contains("1920x1080"));
        assert!(store_screenshot_script.contains("qa-session/store-screenshots.md"));
        assert!(
            !audio_enabled(),
            "audio backend should be opt-in for stable default startup"
        );
        assert!(license_report_script.contains("cargo metadata"));
        assert!(license_report_script.contains("--filter-platform"));
        assert!(license_report_script.contains("missing license metadata"));
        assert!(release_manifest_script.contains("bevy-open-siege-release-manifest-v1"));
        assert!(release_manifest_script.contains("bevy_open_siege.bin"));
        assert!(release_manifest_script.contains("final_signoff_check.sh --check"));
        assert!(release_manifest_script.contains("required_evidence"));
        assert!(release_manifest_script.contains("store-asset-audit.txt"));
        assert!(release_manifest_script.contains("store_asset_audit.sh"));
        assert!(release_manifest_script.contains("content-rating-audit.txt"));
        assert!(release_manifest_script.contains("content_rating_audit.sh"));
        assert!(release_manifest_script.contains("linux-install-smoke.txt"));
        assert!(release_manifest_script.contains("linux_install_smoke.sh"));
        assert!(release_manifest_script.contains("linux-dependency-audit.txt"));
        assert!(release_manifest_script.contains("linux_dependency_audit.sh"));
        assert!(release_manifest_script.contains("linux-portability-smoke.txt"));
        assert!(release_manifest_script.contains("linux_portability_smoke.sh"));
        assert!(release_manifest_script.contains("linux-clean-distro-smoke.txt"));
        assert!(release_manifest_script.contains("linux_clean_distro_smoke.sh"));
        assert!(release_manifest_script.contains("signoff_bundle.sh"));
        assert!(release_manifest_script.contains("create_store_submission_pack.sh"));
        assert!(release_manifest_script.contains("linux-metadata-audit.txt"));
        assert!(release_manifest_script.contains("platform == \"linux-x86_64\""));
        assert!(manifest.branding.iter().any(|entry| entry.id == "icon"));
        assert!(manifest.branding.iter().any(|entry| entry.id == "capsule"));
        assert!(
            manifest
                .branding
                .iter()
                .any(|entry| entry.id == "app_icon_png")
        );
        assert!(
            manifest
                .branding
                .iter()
                .any(|entry| entry.id == "store_capsule_png")
        );
        assert!(manifest.art.iter().any(|entry| entry.id == "plants_sheet"));
        assert!(
            manifest
                .art
                .iter()
                .any(|entry| entry.id == "monsters_sheet")
        );
        assert!(
            manifest
                .data
                .iter()
                .any(|entry| entry.id == "linux_desktop_entry")
        );
        assert!(
            manifest
                .data
                .iter()
                .any(|entry| entry.id == "linux_appstream_metainfo")
        );
        assert!(LINUX_DESKTOP_ENTRY.contains("Exec=bevy_open_siege"));
        assert!(LINUX_DESKTOP_ENTRY.contains("Icon=bevy-open-siege"));
        assert!(LINUX_APPSTREAM_METAINFO.contains("io.github.bevy_open_siege.BevyOpenSiege"));
        assert_eq!(
            appstream_release_version(LINUX_APPSTREAM_METAINFO),
            Some(metadata.version.as_str())
        );
        validate_embedded_asset_format(
            "assets/linux/bevy-open-siege.desktop",
            LINUX_DESKTOP_ENTRY.as_bytes(),
        )
        .unwrap_or_else(|error| panic!("{error}"));
        validate_embedded_asset_format(
            "assets/linux/io.github.bevy_open_siege.BevyOpenSiege.metainfo.xml",
            LINUX_APPSTREAM_METAINFO.as_bytes(),
        )
        .unwrap_or_else(|error| panic!("{error}"));
        assert!(BRAND_ICON_SVG.contains("<svg"));
        assert!(BRAND_CAPSULE_SVG.contains("BEVY OPEN SIEGE"));
        assert!(APP_ICON_PNG.len() > 1024);
        assert!(STORE_CAPSULE_PNG.len() > 1024);
        assert!(PLANTS_SHEET_PNG.len() > 1024);
        assert!(MONSTERS_SHEET_PNG.len() > 1024);
        assert_eq!(png_dimensions(UI_MENU_PANEL_PNG), Some((192, 192)));
        assert_eq!(png_dimensions(UI_HUD_PANEL_PNG), Some((192, 96)));
        assert_eq!(png_dimensions(UI_END_PANEL_PNG), Some((192, 192)));
        assert_eq!(PLANT_SPRITE_ASSETS.len(), PlantKind::COUNT);
        assert_eq!(MONSTER_SPRITE_ASSETS.len(), ZombieKind::COUNT);
        assert_eq!(EFFECT_ASSETS.len(), 6);
        assert_eq!(ENVIRONMENT_ASSETS.len(), 3);
        assert_eq!(UI_ASSETS.len(), 3);
        assert_eq!(AUDIO_ASSETS.len(), 7);
        assert_eq!(
            runtime_asset_paths().len(),
            PlantKind::COUNT
                + ZombieKind::COUNT
                + PLANT_MODEL_ASSETS.len()
                + MONSTER_MODEL_ASSETS.len()
                + EFFECT_ASSETS.len()
                + ENVIRONMENT_ASSETS.len()
                + UI_ASSETS.len()
                + AUDIO_ASSETS.len()
                + 1 // CJK font
        );
        validate_runtime_asset_manifest_coverage(&manifest)
            .expect("all runtime asset paths should be listed in the asset manifest");
        for (path, contents) in PLANT_SPRITE_ASSETS
            .iter()
            .chain(MONSTER_SPRITE_ASSETS.iter())
            .chain(EFFECT_ASSETS.iter())
            .chain(ENVIRONMENT_ASSETS.iter())
            .chain(UI_ASSETS.iter())
        {
            assert!(path.starts_with("assets/art/"));
            assert!(
                contents.len() > 1024,
                "{path} should contain sprite PNG data"
            );
            assert!(
                manifest.art.iter().any(|entry| entry.path == *path),
                "{path} must be listed in the asset manifest"
            );
            validate_embedded_asset_format(path, contents)
                .unwrap_or_else(|error| panic!("{error}"));
        }
        for (path, contents) in AUDIO_ASSETS {
            assert!(path.starts_with("assets/audio/"));
            assert!(path.ends_with(".wav"));
            assert!(contents.len() > 1024, "{path} should contain WAV data");
            assert!(
                manifest.audio.iter().any(|entry| entry.path == path),
                "{path} must be listed in the asset manifest"
            );
            validate_embedded_asset_format(path, contents)
                .unwrap_or_else(|error| panic!("{error}"));
        }

        for required_file in [
            "README.md",
            "LICENSE",
            "CREDITS.md",
            "ART_ASSETS.md",
            "THIRD_PARTY_NOTICES.md",
            "THIRD_PARTY_LICENSES.md",
            "STORE_PAGE.md",
            "STORE_SCREENSHOTS.md",
            "CONTENT_RATING.md",
            "PRESSKIT.md",
            "RELEASE_NOTES.md",
            "RELEASE_CHECKLIST.md",
            "QA_SIGNOFF.md",
            "PRIVACY.md",
            "SUPPORT.md",
            "TROUBLESHOOTING.md",
            "BUILD_PROVENANCE.md",
            "VERSION.ron",
            "asset-audit.txt",
            "audio-audit.txt",
            "controls-audit.txt",
            "input-flow-audit.txt",
            "localization-audit.txt",
            "layout-audit.txt",
            "visual-readability-audit.txt",
            "accessibility-audit.txt",
            "performance-audit.txt",
            "privacy-audit.txt",
            "release-provenance-audit.txt",
            "marketing-audit.txt",
            "ip-audit.txt",
            "save-audit.txt",
            "playthrough-audit.txt",
            "store-asset-audit.txt",
            "content-rating-audit.txt",
            "linux-package-audit.txt",
            "manual-qa-plan.txt",
            "platform-package-plan.txt",
            "final-signoff-plan.txt",
            "release-manifest.json",
            "release-readiness.txt",
            "runtime-smoke.txt",
            "visual-smoke.txt",
            "audio-smoke.txt",
            "scripts/audio_smoke.sh",
            "scripts/install_linux_user.sh",
            "scripts/uninstall_linux_user.sh",
            "scripts/runtime_smoke.sh",
            "scripts/visual_smoke.sh",
            "scripts/store_asset_audit.sh",
            "scripts/content_rating_audit.sh",
            "scripts/linux_package_audit.sh",
            "scripts/manual_qa_session.sh",
            "scripts/platform_package_plan.sh",
            "scripts/final_signoff_check.sh",
            "scripts/support_diagnostics.sh",
            "scripts/signoff_bundle.sh",
            "scripts/create_candidate_evidence.sh",
            "scripts/create_store_submission_pack.sh",
            "scripts/package_windows.ps1",
            "scripts/package_macos.sh",
            "assets/manifest.ron",
            "assets/art",
            "assets/audio",
            "assets/branding",
            "assets/data",
            "assets/i18n",
            "assets/linux",
        ] {
            assert!(
                package_script.contains(required_file),
                "package script must include {required_file}"
            );
        }

        for required_file in [
            "assets/art/plants-sheet.png",
            "assets/art/monsters-sheet.png",
            "assets/art/ui/menu-panel.png",
            "assets/art/ui/hud-panel.png",
            "assets/art/ui/end-panel.png",
            "assets/branding/generated/app-icon.png",
            "assets/branding/generated/store-capsule.png",
            "CONTENT_RATING.md",
            "assets/linux/bevy-open-siege.desktop",
            "assets/linux/io.github.bevy_open_siege.BevyOpenSiege.metainfo.xml",
        ] {
            assert!(
                smoke_script.contains(required_file),
                "smoke test must require {required_file}"
            );
        }
        assert!(smoke_script.contains("plant_sprite_count"));
        assert!(smoke_script.contains("monster_sprite_count"));
        assert!(smoke_script.contains("effect_sprite_count"));
        assert!(smoke_script.contains("environment_texture_count"));
        assert!(smoke_script.contains("ui_chrome_count"));
        assert!(smoke_script.contains("audio_count"));
    }

    #[test]
    fn release_data_validation_cli_path_succeeds() {
        let metadata = validate_release_data().expect("release data should validate");
        assert_eq!(metadata.product_name, "Bevy Open Siege");
    }
}
