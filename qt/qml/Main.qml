import QtQuick
import QtQuick.Controls
import Qt.labs.platform as Platform
import SederDit

ApplicationWindow {
    id: root
    width: 1440
    height: 900
    minimumWidth: 1120
    minimumHeight: 720
    visible: true
    title: "SEDER Media Suite DIT"
    color: Theme.surface.base

    Platform.MenuBar {
        Platform.Menu {
            title: "&File"
            Platform.MenuItem {
                text: "Open Source…"
                shortcut: "Ctrl+O"
                enabled: !appController.busy
                onTriggered: appController.chooseSourceFolder()
            }
            Platform.MenuItem {
                text: "Add Destination…"
                shortcut: "Ctrl+D"
                enabled: !appController.busy
                onTriggered: appController.addDestinationFolder()
            }
            Platform.MenuSeparator {}
            Platform.MenuItem {
                text: "Export TXT Report…"
                shortcut: "Ctrl+E"
                enabled: appController.canExport
                onTriggered: appController.exportTxt()
            }
            Platform.MenuItem {
                text: "Export CSV Report…"
                enabled: appController.canExport
                onTriggered: appController.exportCsv()
            }
            Platform.MenuItem {
                text: "Export MHL Report…"
                enabled: appController.canExportMhl
                onTriggered: appController.exportMhl()
            }
            Platform.MenuItem {
                text: "Export Metadata JSON…"
                enabled: appController.canExportMetadataJson
                onTriggered: appController.exportMetadataJson()
            }
            Platform.MenuItem {
                text: "Export ALE (Avid Log Exchange)…"
                enabled: appController.canExport
                onTriggered: appController.exportAle()
            }
            Platform.MenuSeparator {}
            Platform.MenuItem {
                text: "Quit"
                shortcut: "Ctrl+Q"
                role: Platform.MenuItem.QuitRole
                onTriggered: Qt.quit()
            }
        }
        Platform.Menu {
            title: "&Edit"
            Platform.MenuItem {
                text: "Cancel Offload"
                shortcut: "Esc"
                enabled: appController.busy
                onTriggered: appController.cancelOffload()
            }
            Platform.MenuSeparator {}
            Platform.MenuItem {
                text: "Preferences…"
                shortcut: "Ctrl+,"
                onTriggered: preferencesDialog.open()
            }
        }
        Platform.Menu {
            title: "&Help"
            Platform.MenuItem {
                text: "About SEDER Media Suite DIT"
                role: Platform.MenuItem.AboutRole
                onTriggered: aboutDialog.open()
            }
        }
    }

    Shortcut { sequence: "Ctrl+O"; enabled: !appController.busy; onActivated: appController.chooseSourceFolder() }
    Shortcut { sequence: "Ctrl+D"; enabled: !appController.busy; onActivated: appController.addDestinationFolder() }
    Shortcut { sequence: "Ctrl+E"; enabled: appController.canExport; onActivated: appController.exportTxt() }
    Shortcut { sequence: "Ctrl+,"; onActivated: preferencesDialog.open() }
    Shortcut { sequence: "Ctrl+Q"; onActivated: Qt.quit() }
    Shortcut { sequence: "Escape"; enabled: appController.busy; onActivated: appController.cancelOffload() }

    AboutDialog { id: aboutDialog; anchors.centerIn: parent }
    PreferencesDialog { id: preferencesDialog; anchors.centerIn: parent }

    AppShell {
        anchors.fill: parent
        onOpenPreferences: preferencesDialog.open()
    }
}
