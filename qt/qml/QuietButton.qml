import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import SederDit

Button {
    id: control

    // neutral | primary | danger
    property string variant: "neutral"
    property string iconName: ""

    readonly property color base: variant === "primary" ? Theme.accent.success
        : variant === "danger" ? Theme.accent.brand
        : Theme.surface.raised
    readonly property color hoverColor: variant === "primary" ? Theme.accent.successHi
        : variant === "danger" ? Theme.accent.brandHi
        : Theme.surface.hover
    readonly property color textColor: variant === "neutral"
        ? (control.enabled ? Theme.text.hi : Theme.text.faint)
        : Theme.text.onAccent

    height: Theme.fieldHeight
    font.family: Theme.fontSans
    font.pixelSize: Theme.textBody
    hoverEnabled: true
    focusPolicy: Qt.StrongFocus
    padding: 8

    contentItem: RowLayout {
        spacing: 6
        Item { Layout.fillWidth: true; implicitWidth: 0 }
        Icon {
            visible: control.iconName !== ""
            name: control.iconName
            size: 15
            color: control.textColor
            Layout.alignment: Qt.AlignVCenter
        }
        Text {
            text: control.text
            color: control.textColor
            font: control.font
            horizontalAlignment: Text.AlignHCenter
            verticalAlignment: Text.AlignVCenter
            elide: Text.ElideRight
            Layout.alignment: Qt.AlignVCenter
        }
        Item { Layout.fillWidth: true; implicitWidth: 0 }
    }

    background: Rectangle {
        radius: Theme.radiusSm
        color: !control.enabled
            ? (control.variant === "neutral" ? Theme.surface.raised : Qt.darker(control.base, 1.18))
            : control.down ? Qt.darker(control.base, 1.08)
            : control.hovered ? control.hoverColor
            : control.base
        border.width: control.visualFocus ? Theme.focusRing : 1
        border.color: control.visualFocus
            ? (control.variant === "neutral" ? Theme.accent.brand : control.base)
            : (control.variant === "neutral" ? Theme.border.base : control.base)
        opacity: control.enabled ? 1 : 0.6
        Behavior on color { ColorAnimation { duration: Theme.motionFast } }
    }
}
