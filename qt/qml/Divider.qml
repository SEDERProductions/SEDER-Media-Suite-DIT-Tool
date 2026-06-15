import QtQuick
import SederDit

// Hairline divider. Callers set Layout.fillWidth (horizontal) or
// Layout.fillHeight + vertical:true.
Rectangle {
    property bool vertical: false
    implicitWidth: vertical ? 1 : 0
    implicitHeight: vertical ? 0 : 1
    color: Theme.border.subtle
}
