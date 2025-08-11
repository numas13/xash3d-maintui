// IMPORTANT: only use define_strings macro in this file
//
// The file is included in tools/lang-skeleton.rs.

define_strings! {
    all {
        // common menu items
        BACK = "Back",
        BACK_HINT = "Return to previous menu.",

        // confirm popup
        YES = "Yes",
        CANCEL = "#GameUI_Cancel",

        // quit popup
        QUIT_POPUP_TITLE = "#GameUI_GameMenu_Quit",
        QUIT_POPUP_BODY = "Do you want to exit?",

        // common strings
        TIME = "Time",
        NOW = "Now",
    }
    menu {
        main {
            CONSOLE = "#GameUI_Console",
            CONSOLE_HINT = "Show console.",
            DISCONNECT = "#GameUI_GameMenu_Disconnect",
            DISCONNECT_HINT = "Disconnect from server.",
            RESUME_GAME = "#GameUI_GameMenu_ResumeGame",
            RESUME_GAME_HINT = "Return to game.",
            NEW_GAME = "#GameUI_GameMenu_NewGame",
            NEW_GAME_HINT = "#GameUI_MainMenu_Hint_NewGame",
            NEW_GAME_DEMO = "#GameUI_GameMenu_PlayDemo",
            NEW_GAME_DEMO_HINT = "Start a demo chapter.",
            HAZARD_COURSE = "#GameUI_TrainingRoom",
            HAZARD_COURSE_HINT = "Learn how to play {title}.",
            LOAD_GAME = "#GameUI_GameMenu_LoadGame",
            LOAD_GAME_HINT = "#GameUI_MainMenu_Hint_LoadGame",
            SAVE_GAME = "#GameUI_GameMenu_SaveGame",
            SAVE_GAME_HINT = "Save current game.",
            OPTIONS = "#GameUI_GameMenu_Options",
            OPTIONS_HINT = "#GameUI_MainMenu_Hint_Configuration",
            INTERNET = "Internet servers",
            INTERNET_HINT = "Search for online multiplayer servers on the internet.",
            LAN = "LAN servers",
            LAN_HINT = "Search for online multiplayer servers on the locale area network.",
            CHANGE_GAME = "#GameUI_GameMenu_ChangeGame",
            CHANGE_GAME_HINT = "#GameUI_MainMenu_Hint_ChangeGame",
            QUIT = "#GameUI_GameMenu_Quit",
            QUIT_HINT = "#GameUI_MainMenu_Hint_QuitGame",

            SKILL_EASY = "#GameUI_Easy",
            SKILL_NORMAL = "#GameUI_Medium",
            SKILL_HARD = "#GameUI_Hard",

            DIFFICULTY = "#GameUI_Difficulty",

            DISCONNECT_POPUP = "Do you want to disconnect?",
        }
        save {
            TITLE_LOAD = "#GameUI_GameMenu_LoadGame",
            TITLE_SAVE = "#GameUI_GameMenu_SaveGame",

            DELETE_SAVE = "Delete save",
            NEW_SAVE = "New saved game",

            CONTEXT_TITLE = "Save",

            SAVE_COMMENT = "Comment",
            SAVE_PREVIEW = "Save preview",

            DELETE_POPUP_TITLE = "Delete save",
            DELETE_POPUP_BODY = "Do you want to delete save?",
        }
        browser {
            // title
            TITLE_INTERNET = "Internet servers",
            TITLE_LOCAL = "Local servers",

            // menu
            JOIN_GAME = "Join game",
            CREATE_SERVER = "#GameUI_GameMenu_CreateServer",
            ADD_FAVORITE = "Add favorite server",
            REFRESH = "Refresh",
            SORT = "Sort",

            // tabs
            TAB_DIRECT = "Direct",
            TAB_FAVORITE = "Favorite",
            TAB_NAT = "NAT",

            // list
            COLUMN_HOST = "#GameUI_ServerName",
            COLUMN_MAP = "#GameUI_Map",
            COLUMN_PLAYES = "Players",
            COLUMN_PING = "Ping",

            // sort popup
            SORT_TITLE = "Select sort column",
            SORT_PING = "Ping",
            SORT_NUMCL = "#GameUI_CurrentPlayers",
            SORT_HOST = "#GameUI_ServerName",
            SORT_MAP = "#GameUI_Map",

            // password popup
            PASSWORD_LABEL = "Password:",

            // favorite server address popup
            ADDRESS_LABEL = "Address:",

            // favorite server protocol popup
            PROTOCOL_TITLE = "Select protocol",
            PROTOCOL_XASH3D_49 = "Xash3D 49 (new)",
            PROTOCOL_XASH3D_48 = "Xash3D 48 (old)",
            PROTOCOL_GOLD_SOURCE_48 = "GoldSource 48",
        }
        create_server {
            TITLE = "#GameUI_GameMenu_CreateServer",

            // config list
            START_BUTTON = "Start",
            NAME_LABEL = "#GameUI_ServerName",
            PASSWORD_LABEL = "#GameUI_Password",
            NAT_LABEL = "NAT",
            NAT_HINT = "Use NAT Bypass instead of direct mode",

            // max players popup
            MAX_PLAYERS_TITLE = "#GameUI_MaxPlayers",

            // maps list popup
            MAPS_TITLE = "#GameUI_Map",
            RANDOM_MAP = "#GameUI_RandomMap",
            RANDOM_MAP_TITLE = "No Title",
        }
        config {
            TITLE = "Settings",

            // menu
            KEYBOARD = "Keyboard",
            KEYBOARD_HINT = "Change keyboard settings.",
            GAMEPAD = "Gamepad",
            GAMEPAD_HINT = "Change gamepad settings.",
            MOUSE = "Mouse",
            MOUSE_HINT = "Change mouse settings.",
            GAME = "Game",
            GAME_HINT = "Change game settings.",
            MULTIPLAYER = "Multiplayer",
            MULTIPLAYER_HINT = "Change multiplayer settings.",
            AUDIO = "Audio",
            AUDIO_HINT = "Change audio settings.",
            VOICE = "Voice",
            VOICE_HINT = "Change voice settings.",
            VIDEO = "Video",
            VIDEO_HINT = "Change video settings.",
            NETWORK = "Network",
            NETWORK_HINT = "Change network settings.",
            TOUCH_BUTTONS = "Touch buttons",
            TOUCH_BUTTONS_HINT = "Change touch buttons.",
        }
        config_keyboard {
            TITLE = "Keyboard settings",

            // menu
            RESET = "Reset",

            // table
            COLUMN_ACTION = "Action",
            COLUMN_KEY = "Key/Button",
            COLUMN_KEY_ALT = "Alternate",

            // press key popup
            PRESS_KEY = "Press key or escape to cancel",
        }
        config_mouse {
            TITLE = "Mouse settings",

            // config list
            INVERT_MOUSE = "#GameUI_ReverseMouse",
            MOUSE_LOOK = "#GameUI_MouseLook",
            CROSSHAIR = "Crosshair",
            LOOK_SPRING = "Look spring",
            LOOK_STRAFE = "Look strafe",
            MOUSE_FILTER = "#GameUI_MouseFilter",
            AUTO_AIM = "#GameUI_AutoAim",
            RAW_INPUT = "#GameUI_RawInput",
            AIM_SENSITIVITY = "#GameUI_MouseSensitivity",
        }
        config_gamepad {
            TITLE = "Gamepad settings",

            // config list
            OSC = "Builtin on-screen keyboard",
            SIDE = "Side",
            SIDE_INVERT = "Side invert",
            FORWARD = "Forward",
            FORWARD_INVERT = "Forward invert",
            LOOK_X = "Look X",
            LOOK_X_INVERT = "Look X invert",
            LOOK_Y = "Look Y",
            LOOK_Y_INVERT = "Look Y invert",
            AXIS_BINDINGS_MAP = "Axis binding map",
            AXIS = "Axis",

            // axis pupop
            AXIS_NONE = "NOT BOUND",
            AXIS_SIDE = "Side",
            AXIS_FORWARD = "Forward",
            AXIS_YAW = "Yaw",
            AXIS_PITCH = "Pitch",
            AXIS_LEFT_TRIGGER = "Left Trigger",
            AXIS_RIGHT_TRIGGER = "Right Trigger",
        }
        config_game {
            TITLE = "Game settings",

            // config list (cstrike)
            WEAPON_LAG = "Weapon lag",
        }
        config_multiplayer {
            TITLE = "Multiplayer settings",

            // config list
            PLAYER_NAME = "#GameUI_PlayerName",
            PLAERY_NAME_HINT = "Change the player name.",
            LOGO_TITLE = "Select logo",
            LOGO_LABEL = "Logo",
            LOGO_HINT = "Change the player logo.",
            COLOR_TITLE = "Select color",
            COLOR_LABEL = "Logo color",
            COLOR_HINT = "Change the color of player logo.",
            MODEL_TITLE = "Select model",
            MODEL_LABEL = "Player model",
            MODEL_HINT = "Change the player model.",
            TOP_COLOR = "Top color",
            TOP_COLOR_HINT = "Change the top color of player model.",
            BOTTOM_COLOR = "Bottom color",
            BOTTOM_COLOR_HINT = "Change the bottom color of player model.",
            HIGH_MODELS = "#GameUI_HDModels",
            HIGH_MODELS_HINT = "Use high quality models.",
        }
        config_voice {
            TITLE = "Voice settings",

            // config list
            ENABLE_VOICE = "#GameUI_EnableVoice",
            VOICE_TRANSMIT_VOLUME = "#GameUI_VoiceTransmitVolume",
            VOICE_RECEIVE_VOLUME = "#GameUI_VoiceReceiveVolume",
        }
        config_audio {
            TITLE = "Audio settings",

            // config list
            SOUND_EFFECTS_VOLUME = "#GameUI_SoundEffectVolume",
            MP3_VOLUME = "MP3 volume",
            HEV_SUIT_VOLUME = "#GameUI_HEVSuitVolume",
            SOUND_INTERPOLATION = "Sound interpolation",
            SOUND_INTERPOLATION_DISABLE = "Disable",
            SOUND_INTERPOLATION_BALANCE = "Balance",
            SOUND_INTERPOLATION_QUALITY = "Quality",
            MUTE_INACTIVE = "Mute when inactive",
            DISABLE_DSP_EFFECTS = "Disable DSP effects",
            ALPHA_DSP_EFFECTS = "Use Alpha DSP effects",
            ENABLE_VIBRATION = "Enable vibration",
            VIBRATION = "Vibration",
        }
        config_video {
            TITLE = "Video settings",

            // config list
            GAMMA = "#GameUI_Gamma",
            BRIGHTNESS = "#GameUI_Brightness",
            RESOLUTION = "#GameUI_Resolution",
            WINDOW_MODE = "Window mode",
            WINDOW_MODE_WINDOWED = "Windowed",
            WINDOW_MODE_FULLSCREEN = "Fullscreen",
            WINDOW_MODE_BORDERLESS = "Borderless",
            FPS_LIMIT = "FPS limit",
            FPS_UNLIMITED = "Unlimited",
            VSYNC = "VSync",
            RENDERER = "Renderer",
            RENDERER_NOTE = "loaded",
            DETAIL_TEXTURES = "#GameUI_DetailTextures",
            USE_VBO = "Use VBO",
            WATER_RIPPLES = "Water ripples",
            OVERBRIGHTS = "Overbrights",
            TEXTURE_FILTERING = "Texture filtering",
        }
        config_network {
            TITLE = "Network settings",

            // config list
            ALLOW_DOWNLOAD = "Allow download",

            NETWORK_MODE = "Network mode",
            NETWORK_MODE_SELECT = "Select",
            NETWORK_MODE_NORMAL = "Normal internet connection",
            NETWORK_MODE_DSL = "DSL or PPTP with limited packet size",
            NETWORK_MODE_SLOW = "Slow connection mode (64kbps)",

            NETWORK_SPEED = "Network speed",
            COMMAND_RATE = "Command rate",
            UPDATE_RATE = "Update rate",
        }
        custom_game {
            TITLE = "#GameUI_GameMenu_ChangeGame",

            // table
            COLUMN_TYPE = "#GameUI_Type",
            COLUMN_NAME = "Name",
            COLUMN_VERSION = "Version",
            COLUMN_SIZE = "Size",

            // switch game popup
            CHANGE_POPUP_TITLE = "#GameUI_ChangeGame",
            CHANGE_POPUP_BODY = "Do you want to change game?",
        }
    }
}
