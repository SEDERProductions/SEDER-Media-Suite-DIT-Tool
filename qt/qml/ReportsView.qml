import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import SederDit

// In-app preview of the generated reports with copy / save.
Item {
    id: reportsView

    property int fmt: 0
    readonly property var formats: ["TXT", "CSV", "MHL", "ALE", "JSON"]
    readonly property string content: fmt === 0 ? appController.reportTxt
        : fmt === 1 ? appController.reportCsv
        : fmt === 2 ? appController.reportMhl
        : fmt === 3 ? appController.reportAle
        : appController.reportMetadataJson

    EmptyState {
        anchors.fill: parent
        visible: !appController.canExport
        iconName: "file-text"
        title: "No report yet"
        subtitle: "Run an offload to generate TXT, CSV, MHL, ALE and JSON reports. "
            + "They will appear here for preview, copy and export."
    }

    ColumnLayout {
        anchors.fill: parent
        anchors.margins: Theme.space4
        spacing: Theme.space3
        visible: appController.canExport

        RowLayout {
            Layout.fillWidth: true
            spacing: Theme.space2
            SegmentedControl {
                id: fmtControl
                Layout.preferredWidth: 360
                options: reportsView.formats
                currentIndex: reportsView.fmt
                onActivated: (index) => reportsView.fmt = index
            }
            Item { Layout.fillWidth: true }
            QuietButton {
                text: "Copy"
                iconName: "clipboard"
                enabled: reportsView.content.length > 0
                onClicked: appController.copyText(reportsView.content)
            }
            QuietButton {
                text: "Save…"
                iconName: "duplicate"
                variant: "primary"
                enabled: reportsView.content.length > 0
                onClicked: {
                    switch (reportsView.fmt) {
                    case 0: appController.exportTxt(); break
                    case 1: appController.exportCsv(); break
                    case 2: appController.exportMhl(); break
                    case 3: appController.exportAle(); break
                    case 4: appController.exportMetadataJson(); break
                    }
                }
            }
        }

        Panel {
            Layout.fillWidth: true
            Layout.fillHeight: true
            bodyPadding: Theme.space1

            ScrollView {
                Layout.fillWidth: true
                Layout.fillHeight: true
                Layout.preferredHeight: 320
                clip: true

                TextArea {
                    id: viewer
                    readOnly: true
                    selectByMouse: true
                    wrapMode: TextArea.NoWrap
                    text: reportsView.content.length > 0
                        ? reportsView.content
                        : "This report is not available for the last offload (e.g. MHL needs "
                          + "Verify after copy; JSON needs metadata extraction)."
                    color: reportsView.content.length > 0 ? Theme.text.hi : Theme.text.faint
                    font.family: Theme.fontMono
                    font.pixelSize: Theme.textMeta
                    background: Rectangle { color: Theme.surface.sunken; radius: Theme.radiusSm }
                }
            }
        }
    }
}
