import QtQuick
import QtQuick.Layouts
import SederDit

// Adobe-style segmented toggle. Drop-in for a small ComboBox where the
// options are few and mutually exclusive.
Item {
    id: seg

    property var options: []
    property int currentIndex: 0
    signal activated(int index)

    implicitHeight: Theme.fieldHeight
    implicitWidth: Math.max(160, row.implicitWidth)
    opacity: enabled ? 1 : 0.5

    Rectangle {
        anchors.fill: parent
        radius: Theme.radiusSm
        color: Theme.surface.raised
        border.color: Theme.border.base
    }

    RowLayout {
        id: row
        anchors.fill: parent
        anchors.margins: 2
        spacing: 2
        Repeater {
            model: seg.options
            delegate: Rectangle {
                required property int index
                required property var modelData
                Layout.fillWidth: true
                Layout.fillHeight: true
                radius: Theme.radiusSm - 1
                color: index === seg.currentIndex
                    ? Theme.accent.brand
                    : (hoverArea.containsMouse ? Theme.surface.hover : "transparent")
                Behavior on color { ColorAnimation { duration: Theme.motionFast } }
                Text {
                    anchors.centerIn: parent
                    text: modelData
                    color: index === seg.currentIndex ? Theme.text.onAccent : Theme.text.mid
                    font.family: Theme.fontSans
                    font.pixelSize: Theme.textBody
                    font.bold: index === seg.currentIndex
                }
                MouseArea {
                    id: hoverArea
                    anchors.fill: parent
                    enabled: seg.enabled
                    hoverEnabled: true
                    cursorShape: Qt.PointingHandCursor
                    // Emit only — the owner updates the bound currentIndex so an
                    // external binding (e.g. to a controller property) is preserved.
                    onClicked: if (index !== seg.currentIndex) seg.activated(index)
                }
            }
        }
    }
}
