pragma Singleton

import QtQuick

// Single source of truth for the SEDER design language.
//
// Seeded from the original warm palette and expanded into a proper
// Adobe-grade token system: a tonal surface ramp, semantic accents,
// a spacing scale, type scale, radii, faux-elevation tokens and motion
// tokens. Light/dark is resolved by the existing C++ ThemeController
// (exposed as the `themeController` context property); this singleton
// only derives colors from `themeController.dark`.
//
// Sizes are expressed in logical pixels — Qt 6 performs High-DPI scaling
// automatically, so tokens must NOT be multiplied by devicePixelRatio.
QtObject {
    id: theme

    readonly property bool dark: themeController.dark

    // ---- Surface ramp (deepest backdrop -> most-raised) ------------------
    readonly property QtObject surface: QtObject {
        // App backdrop, sits behind every panel.
        readonly property color backdrop: theme.dark ? "#0d0c0a" : "#e8e1d2"
        // Base window background (= original `bg`).
        readonly property color base:     theme.dark ? "#12110f" : "#ece6d9"
        // Sunken wells (log area, fields).
        readonly property color sunken:   theme.dark ? "#16140f" : "#e3dccb"
        // Standard panel (= original `panel`).
        readonly property color panel:    theme.dark ? "#1f1d1a" : "#f8f4ea"
        // Raised card / panelAlt (= original `panelAlt`).
        readonly property color raised:   theme.dark ? "#282521" : "#e3dccb"
        // Hover / elevated.
        readonly property color hover:    theme.dark ? "#332f2a" : "#ded6c5"
        // Modal scrim.
        readonly property color overlay:  Qt.rgba(0, 0, 0, theme.dark ? 0.55 : 0.32)
    }

    // ---- Borders ---------------------------------------------------------
    readonly property QtObject border: QtObject {
        readonly property color subtle: theme.dark ? "#2c2823" : "#e0d8c7"
        readonly property color base:   theme.dark ? "#3a352e" : "#d6cfbe" // = original `line`
        readonly property color strong: theme.dark ? "#4a443b" : "#c4bba6"
    }

    // ---- Text ------------------------------------------------------------
    readonly property QtObject text: QtObject {
        readonly property color hi:       theme.dark ? "#ece6d9" : "#16140f" // = original `ink`
        readonly property color mid:      theme.dark ? "#ada596" : "#4a4438" // = original `muted`
        readonly property color faint:    theme.dark ? "#716a5f" : "#7a7363" // = original `faint`
        readonly property color onAccent: "#ffffff"
    }

    // ---- Semantic accents (warm SEDER identity preserved) ----------------
    readonly property QtObject accent: QtObject {
        readonly property color brand:    theme.dark ? "#d1411a" : "#c63b13" // = original `red`
        readonly property color brandHi:  theme.dark ? "#e2532d" : "#d84b22"
        readonly property color success:  theme.dark ? "#4cab7e" : "#1f7a4d" // = original `green`
        readonly property color successHi:theme.dark ? "#5ab98d" : "#2f8c5d"
        readonly property color warning:  theme.dark ? "#c99746" : "#9a6a16" // = original `warn`
        readonly property color danger:   theme.dark ? "#d25645" : "#b43a1f" // = original `bad`
        // Tinted status surfaces (originally inline in Main.qml).
        readonly property color successBg: theme.dark ? "#1a2a1f" : "#e0f0e6"
        readonly property color warningBg: theme.dark ? "#2c271b" : "#f4ecd8"
        readonly property color dangerBg:  theme.dark ? "#2c1c19" : "#f4e0da"
    }

    // ---- Spacing (4pt grid) ---------------------------------------------
    function u(n) { return Math.round(n * 4) }
    readonly property int space1: 4
    readonly property int space2: 8
    readonly property int space3: 12
    readonly property int space4: 16
    readonly property int space5: 20
    readonly property int space6: 24

    // ---- Radii -----------------------------------------------------------
    readonly property int radiusSm: 4
    readonly property int radiusMd: 6
    readonly property int radiusLg: 10
    readonly property int focusRing: 2

    // ---- Typography ------------------------------------------------------
    readonly property string fontSans: "Manrope, Inter, Helvetica Neue, Helvetica, Arial, sans-serif"
    readonly property string fontMono: "Menlo, Consolas, monospace"
    readonly property int textCaption: 10
    readonly property int textMeta:    11
    readonly property int textBody:    12
    readonly property int textLabel:   14
    readonly property int textTitle:   18
    readonly property int textDisplay: 24

    // ---- Control metrics -------------------------------------------------
    readonly property int fieldHeight:   32
    readonly property int actionHeight:  38
    readonly property int controlHeight: 28

    // ---- Faux elevation (no QtQuick.Effects on Qt 6.4) -------------------
    // Consumed by Panel/Card via an offset translucent rounded rectangle.
    readonly property color shadowColor: Qt.rgba(0, 0, 0, theme.dark ? 0.45 : 0.16)
    readonly property int elevationOffset: 2
    readonly property int elevationSpread: 1

    // ---- Motion ----------------------------------------------------------
    readonly property int motionFast: 110
    readonly property int motionBase: 170
    readonly property int motionSlow: 260
    readonly property int easeStandard: Easing.OutCubic
    readonly property int easeEmphasized: Easing.OutBack

    // ---- Helpers ---------------------------------------------------------
    function tone(name) {
        if (name === "good") return accent.success
        if (name === "warn") return accent.warning
        if (name === "bad") return accent.danger
        return text.faint
    }
}
