import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

Rectangle {
    id: pathPickerRoot
    property string label: ""
    property string path: ""
    property bool busy: false
    signal pick()
    Layout.fillWidth: true
    height: 68
    color: "transparent"

    readonly property bool dark: themeController.dark
    readonly property color muted: dark ? "#ada596" : "#4a4438"
    readonly property color faint: dark ? "#716a5f" : "#7a7363"
    readonly property color panelAlt: dark ? "#282521" : "#e3dccb"
    readonly property color line: dark ? "#3a352e" : "#d6cfbe"
    readonly property string mono: "Menlo, Consolas, monospace"
    readonly property string sans: "Manrope, Helvetica Neue, Helvetica, Arial, sans-serif"

    ColumnLayout {
        anchors.fill: parent
        spacing: 6
        FieldLabel { text: label }
        RowLayout {
            Layout.fillWidth: true
            spacing: 8
            QuietButton {
                text: "Choose"
                Layout.preferredWidth: 82
                enabled: !busy
                onClicked: pathPickerRoot.pick()
            }
            Rectangle {
                Layout.fillWidth: true
                height: 32
                color: panelAlt
                border.color: line
                radius: 4
                Text {
                    anchors.fill: parent
                    anchors.leftMargin: 8
                    anchors.rightMargin: 8
                    text: path.length > 0 ? path : "No folder selected"
                    color: path.length > 0 ? muted : faint
                    font.family: mono
                    font.pixelSize: 11
                    verticalAlignment: Text.AlignVCenter
                    elide: Text.ElideMiddle
                }
            }
        }
    }
}
