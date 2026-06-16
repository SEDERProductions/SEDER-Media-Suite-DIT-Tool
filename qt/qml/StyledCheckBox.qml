import QtQuick
import QtQuick.Controls
import SederDit

CheckBox {
    id: control
    font.family: Theme.fontSans
    font.pixelSize: Theme.textBody
    spacing: 8
    hoverEnabled: true
    opacity: enabled ? 1 : 0.45

    indicator: Rectangle {
        implicitWidth: 18
        implicitHeight: 18
        x: control.leftPadding
        y: control.topPadding + (control.availableHeight - height) / 2
        radius: Theme.radiusSm
        color: control.checked ? Theme.accent.brand : Theme.surface.raised
        border.color: control.visualFocus ? Theme.accent.brand
            : (control.checked ? Theme.accent.brand
            : (control.hovered ? Theme.text.mid : Theme.border.base))
        border.width: control.visualFocus ? Theme.focusRing : 1
        Behavior on color { ColorAnimation { duration: Theme.motionFast } }

        Icon {
            anchors.centerIn: parent
            name: "check"
            size: 14
            stroke: 2.4
            color: Theme.text.onAccent
            visible: control.checked
        }
    }

    contentItem: Text {
        text: control.text
        font: control.font
        color: control.enabled ? Theme.text.hi : Theme.text.mid
        leftPadding: control.indicator.width + control.spacing
        verticalAlignment: Text.AlignVCenter
        elide: Text.ElideRight
    }
}
