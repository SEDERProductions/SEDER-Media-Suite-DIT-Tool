import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import SederDit

// Application frame: header, left nav rail, mode-switched workspace, status bar.
Item {
    id: shell
    property int mode: 0
    signal openPreferences()

    ColumnLayout {
        anchors.fill: parent
        spacing: 0

        HeaderBar {
            Layout.fillWidth: true
            onOpenSource: appController.chooseSourceFolder()
            onAddDestination: appController.addDestinationFolder()
            onOpenPreferences: shell.openPreferences()
        }

        RowLayout {
            Layout.fillWidth: true
            Layout.fillHeight: true
            spacing: 0

            NavRail {
                Layout.fillHeight: true
                currentMode: shell.mode
                onModeRequested: (m) => shell.mode = m
            }

            StackLayout {
                Layout.fillWidth: true
                Layout.fillHeight: true
                currentIndex: shell.mode

                OffloadView {}

                LibraryView {}

                ReportsView {}

                JobQueueView {}
            }
        }

        StatusBar { Layout.fillWidth: true }
    }
}
