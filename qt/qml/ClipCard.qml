import QtQuick
import QtQuick.Layouts
import SederDit

// Media-library grid cell: thumbnail + filename + meta line.
Rectangle {
    id: card

    property string fileName: ""
    property string metaLine: ""
    property string durationText: ""
    property string mediaKind: ""
    property string thumbSource: ""
    property bool selected: false
    signal clicked()
    signal doubleClicked()

    radius: Theme.radiusMd
    color: Theme.surface.panel
    border.color: selected ? Theme.accent.brand : (hover.containsMouse ? Theme.border.strong : Theme.border.base)
    border.width: selected ? Theme.focusRing : 1
    clip: true

    Behavior on border.color { ColorAnimation { duration: Theme.motionFast } }

    ColumnLayout {
        anchors.fill: parent
        spacing: 0

        // Thumbnail
        Rectangle {
            Layout.fillWidth: true
            Layout.fillHeight: true
            color: Theme.surface.sunken
            clip: true

            Image {
                id: thumb
                anchors.fill: parent
                source: card.thumbSource
                asynchronous: true
                cache: true
                fillMode: Image.PreserveAspectCrop
                visible: status === Image.Ready
            }
            Icon {
                anchors.centerIn: parent
                visible: thumb.status !== Image.Ready
                name: card.mediaKind === "Audio" ? "activity"
                    : (card.mediaKind === "Sidecar" || card.mediaKind === "Subtitle") ? "file-text"
                    : "camera"
                size: 30
                stroke: 1.6
                color: Theme.text.faint
            }

            Rectangle {
                visible: card.durationText !== ""
                anchors.right: parent.right
                anchors.bottom: parent.bottom
                anchors.margins: 6
                radius: 3
                color: Qt.rgba(0, 0, 0, 0.62)
                width: durLabel.implicitWidth + 10
                height: durLabel.implicitHeight + 6
                Text {
                    id: durLabel
                    anchors.centerIn: parent
                    text: card.durationText
                    color: "#ffffff"
                    font.family: Theme.fontMono
                    font.pixelSize: Theme.textCaption
                }
            }
        }

        // Info strip
        ColumnLayout {
            Layout.fillWidth: true
            Layout.margins: Theme.space2
            spacing: 1
            Text {
                Layout.fillWidth: true
                text: card.fileName
                color: Theme.text.hi
                font.family: Theme.fontSans
                font.pixelSize: Theme.textMeta
                font.bold: true
                elide: Text.ElideMiddle
            }
            Text {
                Layout.fillWidth: true
                visible: card.metaLine !== ""
                text: card.metaLine
                color: Theme.text.faint
                font.family: Theme.fontMono
                font.pixelSize: Theme.textCaption
                elide: Text.ElideRight
            }
        }
    }

    MouseArea {
        id: hover
        anchors.fill: parent
        hoverEnabled: true
        cursorShape: Qt.PointingHandCursor
        onClicked: card.clicked()
        onDoubleClicked: card.doubleClicked()
    }
}
