import QtQuick
import QtQuick.Controls

CheckBox {
    id: control
    readonly property bool dark: themeController.dark
    readonly property color ink: dark ? "#ece6d9" : "#16140f"
    readonly property color muted: dark ? "#ada596" : "#4a4438"
    readonly property color faint: dark ? "#716a5f" : "#7a7363"
    readonly property color panelAlt: dark ? "#282521" : "#e3dccb"
    readonly property color line: dark ? "#3a352e" : "#d6cfbe"
    readonly property color red: dark ? "#d1411a" : "#c63b13"
    readonly property string sans: "Manrope, Helvetica Neue, Helvetica, Arial, sans-serif"

    font.family: sans
    font.pixelSize: 12
    spacing: 8
    hoverEnabled: true
    opacity: enabled ? 1 : 0.45
    indicator: Rectangle {
        implicitWidth: 16
        implicitHeight: 16
        x: control.leftPadding
        y: control.topPadding + (control.availableHeight - height) / 2
        radius: 3
        color: control.checked ? red : panelAlt
        border.color: control.visualFocus ? red : (control.hovered ? muted : line)
        border.width: control.visualFocus ? 2 : 1
        Rectangle {
            anchors.centerIn: parent
            width: 8
            height: 8
            radius: 2
            visible: control.checked
            color: "#ffffff"
        }
    }
    contentItem: Text {
        text: control.text
        font: control.font
        color: control.enabled ? ink : muted
        leftPadding: control.indicator.width + control.spacing
        verticalAlignment: Text.AlignVCenter
        elide: Text.ElideRight
    }
}
