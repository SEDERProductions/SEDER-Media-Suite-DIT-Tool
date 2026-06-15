import QtQuick
import QtQuick.Controls
import SederDit

ProgressBar {
    id: control
    from: 0
    to: 1
    hoverEnabled: true
    opacity: enabled ? 1 : 0.45

    background: Rectangle {
        implicitWidth: 180
        implicitHeight: 8
        radius: height / 2
        color: Theme.surface.sunken
        border.color: Theme.border.base
        border.width: 1
    }

    contentItem: Item {
        // Determinate fill.
        Rectangle {
            visible: !control.indeterminate
            width: control.visualPosition * parent.width
            height: parent.height
            radius: height / 2
            color: Theme.accent.brand
            Behavior on width { NumberAnimation { duration: Theme.motionFast } }
        }
        // Indeterminate sweeping chip.
        Item {
            anchors.fill: parent
            visible: control.indeterminate
            clip: true
            Rectangle {
                id: chip
                width: parent.width * 0.35
                height: parent.height
                radius: height / 2
                color: Theme.accent.brand
                x: -width
                XAnimator on x {
                    from: -chip.width
                    to: control.width
                    duration: Theme.motionSlow * 4
                    loops: Animation.Infinite
                    running: control.indeterminate && control.visible
                }
            }
        }
    }
}
