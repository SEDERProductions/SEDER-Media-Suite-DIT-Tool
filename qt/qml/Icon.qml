import QtQuick
import QtQuick.Shapes
import SederDit

// Themeable vector icon rendered with QtQuick.Shapes (no icon font, no
// effects module — works on Qt 6.4 and recolors with the theme).
// Paths are authored in a 24x24 viewBox and scaled to `size`.
Item {
    id: root

    property string name: ""
    property color color: Theme.text.mid
    property int size: 16
    // Desired on-screen stroke width in px (kept constant across sizes).
    property real stroke: 2

    implicitWidth: size
    implicitHeight: size

    readonly property var _icons: ({
        "dot":          { d: "M12 9 A3 3 0 1 1 12 15 A3 3 0 1 1 12 9", fill: true },
        "check":        { d: "M4 12.5 L9.5 18 L20 6.5" },
        "x":            { d: "M6 6 L18 18 M18 6 L6 18" },
        "plus":         { d: "M12 5 V19 M5 12 H19" },
        "minus":        { d: "M5 12 H19" },
        "chevron-right":{ d: "M9 5 L16 12 L9 19" },
        "chevron-left": { d: "M15 5 L8 12 L15 19" },
        "chevron-down": { d: "M5 9 L12 16 L19 9" },
        "chevron-up":   { d: "M5 15 L12 8 L19 15" },
        "arrow-right":  { d: "M4 12 H20 M13 5 L20 12 L13 19" },
        "folder":       { d: "M3 7 H9 L11 9 H21 V19 H3 Z" },
        "folder-plus":  { d: "M3 7 H9 L11 9 H21 V19 H3 Z M12 12 V16 M10 14 H14" },
        "copy":         { d: "M8 8 H19 V19 H8 Z M4 16 V4 H16" },
        "duplicate":    { d: "M7 17 L17 7 M9 7 H17 V15" },
        "circle":       { d: "M3 12 A9 9 0 1 1 21 12 A9 9 0 1 1 3 12" },
        "check-circle": { d: "M3 12 A9 9 0 1 1 21 12 A9 9 0 1 1 3 12 M8 12 L11 15 L16 8.5" },
        "x-circle":     { d: "M3 12 A9 9 0 1 1 21 12 A9 9 0 1 1 3 12 M9 9 L15 15 M15 9 L9 15" },
        "slash-circle": { d: "M3 12 A9 9 0 1 1 21 12 A9 9 0 1 1 3 12 M6.5 6.5 L17.5 17.5" },
        "info":         { d: "M3 12 A9 9 0 1 1 21 12 A9 9 0 1 1 3 12 M12 11 V16.5 M12 7.5 V8" },
        "alert":        { d: "M12 4 L21.5 20 H2.5 Z M12 10 V14.5 M12 17.5 V17.8" },
        "octagon":      { d: "M8.5 3 H15.5 L21 8.5 V15.5 L15.5 21 H8.5 L3 15.5 V8.5 Z M9 9 L15 15 M15 9 L9 15" },
        "activity":     { d: "M3 12 H8 L10 6 L14 18 L16 12 H21" },
        "play":         { d: "M7 5 L19 12 L7 19 Z", fill: true },
        "pause":        { d: "M8 5 V19 M16 5 V19" },
        "stop":         { d: "M6 6 H18 V18 H6 Z", fill: true },
        "list":         { d: "M8 6 H21 M8 12 H21 M8 18 H21 M3.5 6 H4 M3.5 12 H4 M3.5 18 H4" },
        "grid":         { d: "M4 4 H10 V10 H4 Z M14 4 H20 V10 H14 Z M4 14 H10 V20 H4 Z M14 14 H20 V20 H14 Z" },
        "file-text":    { d: "M6 3 H14 L19 8 V21 H6 Z M14 3 V8 H19 M9 13 H16 M9 17 H16" },
        "hard-drive":   { d: "M5 7 H19 L21 13 V18 H3 V13 Z M3 13 H21 M7 15.5 H7.3 M10 15.5 H10.3" },
        "camera":       { d: "M4 8 H7 L9 5 H15 L17 8 H20 V19 H4 Z M12 11 A3 3 0 1 1 12 17 A3 3 0 1 1 12 11" },
        "calendar":     { d: "M4 6 H20 V20 H4 Z M4 10 H20 M8 4 V8 M16 4 V8" },
        "refresh":      { d: "M20 6 V11 H15 M19.5 11 A8 8 0 1 0 19 15.5" },
        "trash":        { d: "M4 7 H20 M9 7 V5 H15 V7 M6 7 V20 H18 V7 M10 11 V16 M14 11 V16" },
        "clipboard":    { d: "M9 4 H15 V6 H9 Z M9 5 H6 V20 H18 V5 H15 M9 11 H15 M9 15 H15" },
        "sliders":      { d: "M4 8 H20 M4 16 H20 M9 8 A1.6 1.6 0 1 1 9 5 A1.6 1.6 0 1 1 9 8 M15 19 A1.6 1.6 0 1 1 15 16 A1.6 1.6 0 1 1 15 19" },
        "search":       { d: "M4 11 A7 7 0 1 1 18 11 A7 7 0 1 1 4 11 M16 16 L21 21" },
        "layers":       { d: "M12 4 L21 9 L12 14 L3 9 Z M3 14 L12 19 L21 14" },
        "drop":         { d: "M12 4 L18 12 A6 6 0 1 1 6 12 Z" }
    })

    readonly property var _def: (name !== "" && _icons[name] !== undefined) ? _icons[name] : _icons["dot"]

    Shape {
        id: shape
        anchors.fill: parent
        antialiasing: true
        ShapePath {
            strokeColor: root.color
            // Constant on-screen stroke regardless of size (paths are in 24-space).
            strokeWidth: root.stroke * 24 / Math.max(1, root.size)
            fillColor: (root._def.fill === true) ? root.color : "transparent"
            capStyle: ShapePath.RoundCap
            joinStyle: ShapePath.RoundJoin
            PathSvg { path: root._def.d }
        }
        transform: Scale {
            origin.x: 0; origin.y: 0
            xScale: root.size / 24
            yScale: root.size / 24
        }
    }
}
