import QtQuick
import QtQuick.Controls

ProgressBar {
    id: control
    readonly property bool dark: themeController.dark
    readonly property color panelAlt: dark ? "#282521" : "#e3dccb"
    readonly property color line: dark ? "#3a352e" : "#d6cfbe"
    readonly property color red: dark ? "#d1411a" : "#c63b13"

    from: 0
    to: 1
    hoverEnabled: true
    opacity: enabled ? 1 : 0.45
    background: Rectangle {
        implicitWidth: 180
        implicitHeight: 8
        radius: 4
        color: panelAlt
        border.color: control.visualFocus ? red : (control.hovered ? Qt.darker(line, 1.2) : line)
        border.width: control.visualFocus ? 2 : 1
    }
    contentItem: Item {
        Rectangle {
            width: control.visualPosition * parent.width
            height: parent.height
            radius: 4
            color: red
        }
    }
}
