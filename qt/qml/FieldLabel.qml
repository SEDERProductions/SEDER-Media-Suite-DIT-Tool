import QtQuick

Text {
    readonly property bool dark: themeController.dark
    readonly property color muted: dark ? "#ada596" : "#4a4438"
    readonly property string sans: "Manrope, Helvetica Neue, Helvetica, Arial, sans-serif"

    color: muted
    font.family: sans
    font.pixelSize: 12
}
