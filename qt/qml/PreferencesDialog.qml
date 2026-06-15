import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import SederDit

Dialog {
    id: prefsDialog
    modal: true
    title: "Preferences"
    standardButtons: Dialog.Close

    background: Rectangle {
        color: Theme.surface.panel
        border.color: Theme.border.strong
        radius: Theme.radiusMd
    }

    component SectionTitle: Text {
        color: Theme.text.hi
        font.family: Theme.fontSans
        font.pixelSize: Theme.textLabel
        font.bold: true
    }
    component HelpText: Text {
        Layout.fillWidth: true
        wrapMode: Text.WordWrap
        color: Theme.text.faint
        font.family: Theme.fontSans
        font.pixelSize: Theme.textMeta
    }

    ColumnLayout {
        spacing: Theme.space4
        width: 500

        SectionTitle { text: "Appearance" }
        RowLayout {
            Layout.fillWidth: true
            spacing: Theme.space2
            FieldLabel { text: "Theme" }
            StyledComboBox {
                Layout.fillWidth: true
                model: ["system", "light", "dark"]
                currentIndex: Math.max(0, model.indexOf(themeController.preference))
                onActivated: themeController.preference = model[currentIndex]
            }
        }

        Divider { Layout.fillWidth: true }

        SectionTitle { text: "Default offload options" }
        HelpText { text: "These values are used the next time the app starts." }

        StyledCheckBox {
            text: "Ignore hidden / system files by default"
            checked: settingsStore.defaultIgnoreHiddenSystem
            onToggled: settingsStore.defaultIgnoreHiddenSystem = checked
        }
        StyledCheckBox {
            text: "Verify after copy by default"
            checked: settingsStore.defaultVerifyAfterCopy
            onToggled: settingsStore.defaultVerifyAfterCopy = checked
        }
        StyledCheckBox {
            text: "Skip files already present at destination"
            checked: settingsStore.defaultSkipExisting
            onToggled: settingsStore.defaultSkipExisting = checked
        }
        StyledCheckBox {
            text: "Generate report by default"
            checked: settingsStore.defaultGenerateReport
            onToggled: settingsStore.defaultGenerateReport = checked
        }

        ColumnLayout {
            Layout.fillWidth: true
            spacing: 4
            FieldLabel { text: "Default ignore patterns (comma-separated)" }
            DenseTextField {
                Layout.fillWidth: true
                text: settingsStore.defaultIgnorePatterns
                font.family: Theme.fontMono
                font.pixelSize: Theme.textMeta
                onEditingFinished: settingsStore.defaultIgnorePatterns = text
            }
        }

        RowLayout {
            Layout.fillWidth: true
            spacing: Theme.space2
            FieldLabel { text: "Default checksum algorithm" }
            StyledComboBox {
                id: algoCombo
                readonly property var algos: ["BLAKE3", "MD5", "SHA1", "XXH3-64", "XXH3-128"]
                Layout.fillWidth: true
                model: algos
                currentIndex: Math.max(0, algos.indexOf(settingsStore.defaultChecksumAlgorithm))
                onActivated: settingsStore.defaultChecksumAlgorithm = algos[currentIndex]
            }
        }
        HelpText {
            text: "BLAKE3 (default) is fast and cryptographic. XXH3-64 is fastest and good for "
                + "in-house verification. MD5 / SHA-1 are slower but interoperate with legacy DIT pipelines."
        }

        Divider { Layout.fillWidth: true }

        SectionTitle { text: "Destination subfolder template" }
        HelpText {
            text: "Optional. When set, picking a destination folder will create and use the "
                + "expanded subfolder underneath it. Tokens: {project}, {date}, {card}, {camera}."
        }
        DenseTextField {
            id: templateField
            Layout.fillWidth: true
            text: settingsStore.destinationTemplate
            placeholderText: "{project}/{date}/{card}"
            font.family: Theme.fontMono
            font.pixelSize: Theme.textMeta
            onEditingFinished: settingsStore.destinationTemplate = text
        }
        Text {
            id: templatePreview
            Layout.fillWidth: true
            wrapMode: Text.WrapAnywhere
            text: "Preview: " + (settingsStore.destinationTemplate.length > 0
                ? appController.previewDestinationTemplate("")
                : "(no template — destination folder used as-is)")
            color: Theme.text.mid
            font.family: Theme.fontMono
            font.pixelSize: Theme.textMeta
        }

        Divider { Layout.fillWidth: true }

        SectionTitle { text: "Clip metadata extraction" }
        StyledCheckBox {
            text: "Extract clip metadata with ffprobe during scan"
            enabled: appController.ffprobeAvailable
            checked: settingsStore.defaultExtractMetadata && appController.ffprobeAvailable
            onToggled: settingsStore.defaultExtractMetadata = checked
        }
        Text {
            Layout.fillWidth: true
            wrapMode: Text.WordWrap
            text: appController.ffprobeAvailable
                ? "ffprobe was found on this system. Extracting metadata makes scans slower but "
                  + "populates the Library and report sidecars with codec, resolution, frame rate, "
                  + "duration, audio, color space, and timecode for each clip."
                : "ffprobe was not found on this system. Install FFmpeg (brew install ffmpeg, "
                  + "apt-get install ffmpeg, or https://www.ffmpeg.org/download.html) and relaunch "
                  + "the app to enable this option."
            color: appController.ffprobeAvailable ? Theme.text.faint : Theme.accent.warning
            font.family: Theme.fontSans
            font.pixelSize: Theme.textMeta
        }

        RowLayout {
            Layout.fillWidth: true
            spacing: Theme.space2
            Item { Layout.fillWidth: true }
            QuietButton {
                text: "Reset to factory defaults"
                onClicked: settingsStore.resetDefaultsToFactory()
            }
            QuietButton {
                text: "Apply to current session"
                variant: "primary"
                onClicked: {
                    appController.applyDefaultsFromSettings()
                    prefsDialog.close()
                }
            }
        }
    }
}
