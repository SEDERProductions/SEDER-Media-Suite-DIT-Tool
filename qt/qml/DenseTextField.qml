import QtQuick
import QtQuick.Controls
import SederDit

TextField {
    id: field
    color: Theme.text.hi
    selectedTextColor: Theme.text.onAccent
    selectionColor: Theme.accent.brand
    font.family: Theme.fontSans
    font.pixelSize: Theme.textBody
    padding: 8
    placeholderTextColor: Theme.text.faint
    background: Rectangle {
        radius: Theme.radiusSm
        color: Theme.surface.raised
        border.color: field.activeFocus ? Theme.accent.brand : Theme.border.base
        border.width: field.activeFocus ? Theme.focusRing : 1
        Behavior on border.color { ColorAnimation { duration: Theme.motionFast } }
    }
}
