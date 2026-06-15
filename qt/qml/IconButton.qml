import QtQuick
import QtQuick.Controls
import SederDit

// Compact square button that renders a single themeable Icon.
Button {
    id: control

    property string iconName: ""
    property int iconSize: 16
    property color iconColor: Theme.text.mid
    // quiet | solid | danger
    property string variant: "quiet"

    implicitWidth: Theme.controlHeight
    implicitHeight: Theme.controlHeight
    padding: 0
    hoverEnabled: true
    focusPolicy: Qt.StrongFocus

    readonly property color _resolvedIconColor: !control.enabled
        ? Theme.text.faint
        : variant === "solid" ? Theme.text.onAccent
        : variant === "danger" ? (control.hovered ? Theme.accent.danger : Theme.text.mid)
        : (control.hovered ? Theme.text.hi : iconColor)

    contentItem: Item {
        Icon {
            anchors.centerIn: parent
            name: control.iconName
            size: control.iconSize
            color: control._resolvedIconColor
        }
    }

    background: Rectangle {
        radius: Theme.radiusSm
        color: {
            if (control.variant === "solid")
                return control.down ? Theme.accent.brandHi
                     : control.hovered ? Theme.accent.brandHi : Theme.accent.brand
            if (!control.enabled) return "transparent"
            if (control.down) return Theme.surface.hover
            if (control.hovered) return Theme.surface.raised
            return "transparent"
        }
        border.width: control.visualFocus ? Theme.focusRing : 0
        border.color: Theme.accent.brand
        Behavior on color { ColorAnimation { duration: Theme.motionFast } }
    }
}
