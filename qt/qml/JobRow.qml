import QtQuick
import QtQuick.Layouts
import SederDit

// One row in the offload job queue.
Rectangle {
    id: row

    property string label: ""
    property string sourcePath: ""
    property int destinationCount: 0
    property int state: 0        // 0 Queued, 1 Running, 2 Complete, 3 Failed, 4 Cancelled
    property double progress: 0
    property string finalStatus: ""
    property int totalFiles: 0
    property bool busy: false
    property bool canMoveUp: false
    property bool canMoveDown: false
    signal removeRequested()
    signal moveUpRequested()
    signal moveDownRequested()

    implicitHeight: 68
    radius: Theme.radiusMd
    color: Theme.surface.panel
    border.color: state === 1 ? Theme.accent.brand : Theme.border.base

    function stateIcon(s) {
        return s === 1 ? "activity" : s === 2 ? "check-circle"
             : s === 3 ? "x-circle" : s === 4 ? "slash-circle" : "circle"
    }
    function stateTone(s) {
        return s === 2 ? "good" : s === 3 ? "bad" : s === 1 ? "warn" : "neutral"
    }
    function stateLabel(s) {
        return s === 0 ? "Queued" : s === 1 ? "Running"
             : s === 2 ? "Complete" : s === 3 ? "Failed" : "Cancelled"
    }

    RowLayout {
        anchors.fill: parent
        anchors.margins: Theme.space3
        spacing: Theme.space3

        Icon {
            name: row.stateIcon(row.state)
            size: 22
            color: row.stateTone(row.state) === "good" ? Theme.accent.success
                : row.stateTone(row.state) === "bad" ? Theme.accent.danger
                : row.stateTone(row.state) === "warn" ? Theme.accent.warning
                : Theme.text.faint
            Layout.alignment: Qt.AlignVCenter
        }

        ColumnLayout {
            Layout.fillWidth: true
            spacing: 2
            RowLayout {
                Layout.fillWidth: true
                spacing: Theme.space2
                Text {
                    text: row.label
                    color: Theme.text.hi
                    font.family: Theme.fontSans
                    font.pixelSize: Theme.textBody
                    font.bold: true
                    elide: Text.ElideMiddle
                    Layout.fillWidth: true
                }
                StatusPill { text: row.stateLabel(row.state); tone: row.stateTone(row.state) }
            }
            Text {
                Layout.fillWidth: true
                text: row.finalStatus !== "" ? (row.finalStatus + " · " + row.totalFiles + " files") : row.sourcePath
                color: Theme.text.faint
                font.family: Theme.fontMono
                font.pixelSize: Theme.textCaption
                elide: Text.ElideMiddle
            }
            StyledProgressBar {
                Layout.fillWidth: true
                from: 0; to: 1
                value: row.progress
                visible: row.state === 1
            }
        }

        IconButton {
            iconName: "chevron-up"
            enabled: row.canMoveUp && !row.busy
            Accessible.name: "Move job up"
            onClicked: row.moveUpRequested()
        }
        IconButton {
            iconName: "chevron-down"
            enabled: row.canMoveDown && !row.busy
            Accessible.name: "Move job down"
            onClicked: row.moveDownRequested()
        }
        IconButton {
            iconName: "x"
            variant: "danger"
            enabled: !row.busy && row.state !== 1
            Accessible.name: "Remove job"
            onClicked: row.removeRequested()
        }
    }
}
