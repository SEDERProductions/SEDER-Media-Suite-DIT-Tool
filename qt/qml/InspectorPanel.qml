import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import SederDit

// Right-hand clip inspector: poster, grouped metadata, proxy action.
Rectangle {
    id: inspector

    property var clip: null
    property string sourcePath: ""

    color: Theme.surface.panel

    readonly property string thumbSource: (clip && clip.hash && String(clip.hash).length > 0 && sourcePath.length > 0)
        ? "image://clipthumb/" + clip.algorithm + "/" + clip.hash + "/"
            + encodeURIComponent(sourcePath + "/" + clip.relPath)
        : ""

    Divider { anchors.left: parent.left; anchors.top: parent.top; anchors.bottom: parent.bottom; vertical: true }

    // Empty state when nothing is selected.
    EmptyState {
        anchors.fill: parent
        visible: !inspector.clip
        iconName: "info"
        title: "No clip selected"
        subtitle: "Select a clip in the library to inspect its metadata."
    }

    ScrollView {
        anchors.fill: parent
        anchors.leftMargin: 1
        visible: inspector.clip
        contentWidth: availableWidth
        clip: true

        ColumnLayout {
            width: inspector.width - 1
            spacing: Theme.space3

            // Poster
            Rectangle {
                Layout.fillWidth: true
                Layout.margins: Theme.space3
                Layout.bottomMargin: 0
                Layout.preferredHeight: width * 9 / 16
                radius: Theme.radiusMd
                color: Theme.surface.sunken
                border.color: Theme.border.base
                clip: true
                Image {
                    id: poster
                    anchors.fill: parent
                    source: inspector.thumbSource
                    asynchronous: true
                    cache: true
                    fillMode: Image.PreserveAspectCrop
                    visible: status === Image.Ready
                }
                Icon {
                    anchors.centerIn: parent
                    visible: poster.status !== Image.Ready
                    name: (inspector.clip && inspector.clip.mediaKind === "Audio") ? "activity" : "camera"
                    size: 40
                    stroke: 1.5
                    color: Theme.text.faint
                }
            }

            ColumnLayout {
                Layout.fillWidth: true
                Layout.leftMargin: Theme.space3
                Layout.rightMargin: Theme.space3
                spacing: Theme.space1

                Text {
                    Layout.fillWidth: true
                    text: inspector.clip ? inspector.clip.fileName : ""
                    color: Theme.text.hi
                    font.family: Theme.fontSans
                    font.pixelSize: Theme.textLabel
                    font.bold: true
                    wrapMode: Text.WrapAnywhere
                }
                StatusPill {
                    visible: inspector.clip && String(inspector.clip.mediaKind).length > 0
                    text: inspector.clip ? inspector.clip.mediaKind : ""
                    tone: "info"
                }
            }

            Divider { Layout.fillWidth: true; Layout.leftMargin: Theme.space3; Layout.rightMargin: Theme.space3 }

            ColumnLayout {
                Layout.fillWidth: true
                Layout.leftMargin: Theme.space3
                Layout.rightMargin: Theme.space3
                spacing: 6
                MetaRow { Layout.fillWidth: true; label: "Codec"; value: inspector.clip ? inspector.clip.codec : "" }
                MetaRow { Layout.fillWidth: true; label: "Resolution"; value: inspector.clip ? inspector.clip.resolution : "" }
                MetaRow { Layout.fillWidth: true; label: "Frame rate"; value: inspector.clip ? inspector.clip.fps : "" }
                MetaRow { Layout.fillWidth: true; label: "Duration"; value: inspector.clip ? inspector.clip.duration : "" }
                MetaRow { Layout.fillWidth: true; label: "Timecode"; value: inspector.clip ? inspector.clip.timecode : "" }
                MetaRow { Layout.fillWidth: true; label: "Color"; value: inspector.clip ? inspector.clip.colorSpace : "" }
                MetaRow { Layout.fillWidth: true; label: "Audio"; value: inspector.clip ? inspector.clip.audio : "" }
                MetaRow { Layout.fillWidth: true; label: "Size"; value: inspector.clip ? inspector.clip.sizeText : "" }
                MetaRow { Layout.fillWidth: true; label: "Path"; value: inspector.clip ? inspector.clip.relPath : "" }
                MetaRow { Layout.fillWidth: true; label: inspector.clip ? inspector.clip.algorithm : "Hash"; value: inspector.clip ? inspector.clip.hash : "" }
            }

            Divider { Layout.fillWidth: true; Layout.leftMargin: Theme.space3; Layout.rightMargin: Theme.space3 }

            // Proxy generation
            ColumnLayout {
                Layout.fillWidth: true
                Layout.leftMargin: Theme.space3
                Layout.rightMargin: Theme.space3
                Layout.bottomMargin: Theme.space4
                spacing: Theme.space2
                MetaLabel { text: "Proxy" }
                RowLayout {
                    Layout.fillWidth: true
                    spacing: Theme.space2
                    StyledComboBox {
                        id: presetCombo
                        Layout.fillWidth: true
                        model: ["PRORES", "H264", "DNXHR"]
                        enabled: appController.ffmpegAvailable
                    }
                    QuietButton {
                        text: "Generate"
                        iconName: "play"
                        variant: "primary"
                        enabled: appController.ffmpegAvailable && inspector.clip
                        onClicked: appController.generateProxy(inspector.clip.relPath, presetCombo.currentText)
                    }
                }
                Text {
                    Layout.fillWidth: true
                    visible: !appController.ffmpegAvailable
                    text: "Install FFmpeg to generate proxies."
                    color: Theme.accent.warning
                    font.family: Theme.fontSans
                    font.pixelSize: Theme.textMeta
                    wrapMode: Text.WordWrap
                }
            }
        }
    }
}
