import QtQuick

Text {
    readonly property bool dark: themeController.dark
    readonly property color faint: dark ? "#716a5f" : "#7a7363"
    readonly property string mono: "Menlo, Consolas, monospace"

    color: faint
    font.family: mono
    font.pixelSize: 10
    font.capitalization: Font.AllUppercase
}
