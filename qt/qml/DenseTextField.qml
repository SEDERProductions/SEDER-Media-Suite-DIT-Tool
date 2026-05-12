import QtQuick
import QtQuick.Controls

TextField {
    readonly property bool dark: themeController.dark
    readonly property color ink: dark ? "#ece6d9" : "#16140f"
    readonly property color faint: dark ? "#716a5f" : "#7a7363"
    readonly property color panelAlt: dark ? "#282521" : "#e3dccb"
    readonly property color line: dark ? "#3a352e" : "#d6cfbe"
    readonly property color red: dark ? "#d1411a" : "#c63b13"
    readonly property string sans: "Manrope, Helvetica Neue, Helvetica, Arial, sans-serif"

    color: ink
    selectedTextColor: "#ffffff"
    selectionColor: red
    font.family: sans
    font.pixelSize: 13
    padding: 8
    placeholderTextColor: faint
    background: Rectangle {
        color: panelAlt
        border.color: line
        radius: 4
    }
}
