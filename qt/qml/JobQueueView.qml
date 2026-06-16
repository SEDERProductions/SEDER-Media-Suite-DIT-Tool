import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import SederDit

// Job queue dashboard: stage and run multiple offloads serially.
Item {
    id: queueView

    ColumnLayout {
        anchors.fill: parent
        spacing: 0

        // Toolbar
        Rectangle {
            Layout.fillWidth: true
            Layout.preferredHeight: 52
            color: Theme.surface.panel
            RowLayout {
                anchors.fill: parent
                anchors.leftMargin: Theme.space4
                anchors.rightMargin: Theme.space4
                spacing: Theme.space2
                Icon { name: "list"; size: 15; color: Theme.text.mid }
                MetaLabel { text: "Job Queue" }
                Text {
                    text: appController.jobQueue.count > 0
                        ? "· " + appController.jobQueue.activeCount + " pending of " + appController.jobQueue.count
                        : ""
                    color: Theme.text.faint
                    font.family: Theme.fontMono
                    font.pixelSize: Theme.textCaption
                }
                Item { Layout.fillWidth: true }
                QuietButton {
                    text: "Clear Finished"
                    iconName: "trash"
                    enabled: !appController.busy && appController.jobQueue.count > appController.jobQueue.activeCount
                    onClicked: appController.jobQueue.clearFinished()
                }
                QuietButton {
                    text: appController.busy ? "Running…" : "Run Queue"
                    iconName: appController.busy ? "activity" : "play"
                    variant: "primary"
                    enabled: !appController.busy && appController.jobQueue.activeCount > 0
                    onClicked: appController.runQueue()
                }
            }
            Divider { anchors.left: parent.left; anchors.right: parent.right; anchors.bottom: parent.bottom }
        }

        EmptyState {
            Layout.fillWidth: true
            Layout.fillHeight: true
            visible: appController.jobQueue.count === 0
            iconName: "list"
            title: "The queue is empty"
            subtitle: "Configure an offload, then use “Add to Queue” to stage multiple jobs and "
                + "run them back-to-back. “Start Offload” also queues and runs immediately."
        }

        ListView {
            id: list
            Layout.fillWidth: true
            Layout.fillHeight: true
            Layout.margins: Theme.space4
            visible: appController.jobQueue.count > 0
            clip: true
            spacing: Theme.space2
            model: appController.jobQueue
            ScrollBar.vertical: ScrollBar {}

            delegate: JobRow {
                required property int index
                required property var model
                width: ListView.view ? ListView.view.width : 0
                label: model.label
                sourcePath: model.sourcePath
                destinationCount: model.destinationCount
                state: model.state
                progress: model.progress
                finalStatus: model.finalStatus
                totalFiles: model.totalFiles
                busy: appController.busy
                canMoveUp: index > 0
                canMoveDown: index < appController.jobQueue.count - 1
                onRemoveRequested: appController.jobQueue.removeJob(index)
                onMoveUpRequested: appController.jobQueue.moveJob(index, index - 1)
                onMoveDownRequested: appController.jobQueue.moveJob(index, index + 1)
            }
        }
    }
}
