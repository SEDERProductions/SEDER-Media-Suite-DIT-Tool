import QtQuick
import QtQuick.Layouts
import SederDit

// Titled surface card with faux elevation (no QtQuick.Effects on Qt 6.4)
// and optional collapse. Children are placed in a ColumnLayout body.
Item {
    id: panel

    property string title: ""
    property string icon: ""
    property string kicker: ""          // small mono label above content (e.g. "01 / Source")
    property bool collapsible: false
    property bool collapsed: false
    property bool elevated: true
    property int bodyPadding: Theme.space4
    property alias bodySpacing: body.spacing
    default property alias content: body.data

    implicitWidth: 240
    implicitHeight: frame.implicitHeight

    // Faux shadow: offset translucent rounded rect behind the frame.
    Rectangle {
        visible: panel.elevated
        x: 0
        y: Theme.elevationOffset
        width: parent.width
        height: frame.height
        radius: Theme.radiusMd
        color: Theme.shadowColor
        z: -1
    }

    Rectangle {
        id: frame
        width: parent.width
        implicitHeight: layout.implicitHeight
        radius: Theme.radiusMd
        color: Theme.surface.panel
        border.color: Theme.border.base
        clip: true

        ColumnLayout {
            id: layout
            width: parent.width
            spacing: 0

            // ---- Header ----
            Item {
                Layout.fillWidth: true
                Layout.preferredHeight: panel.title !== "" ? 38 : 0
                visible: panel.title !== ""

                RowLayout {
                    anchors.fill: parent
                    anchors.leftMargin: Theme.space4
                    anchors.rightMargin: Theme.space2
                    spacing: Theme.space2

                    Icon {
                        visible: panel.icon !== ""
                        name: panel.icon
                        size: 15
                        color: Theme.text.mid
                        Layout.alignment: Qt.AlignVCenter
                    }
                    Text {
                        text: panel.title
                        color: Theme.text.hi
                        font.family: Theme.fontSans
                        font.pixelSize: Theme.textBody
                        font.bold: true
                        font.capitalization: Font.AllUppercase
                        font.letterSpacing: 0.5
                        Layout.fillWidth: true
                        Layout.alignment: Qt.AlignVCenter
                        elide: Text.ElideRight
                    }
                    IconButton {
                        visible: panel.collapsible
                        iconName: panel.collapsed ? "chevron-down" : "chevron-up"
                        iconSize: 16
                        Accessible.name: panel.collapsed ? "Expand " + panel.title : "Collapse " + panel.title
                        onClicked: panel.collapsed = !panel.collapsed
                    }
                }
            }

            Divider { Layout.fillWidth: true; visible: panel.title !== "" && !panel.collapsed }

            // ---- Body ----
            ColumnLayout {
                id: body
                Layout.fillWidth: true
                Layout.margins: panel.bodyPadding
                Layout.topMargin: panel.title !== "" ? panel.bodyPadding : panel.bodyPadding
                spacing: Theme.space3
                visible: !panel.collapsed
            }
        }
    }
}
