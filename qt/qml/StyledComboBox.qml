import QtQuick
import QtQuick.Controls
import SederDit

ComboBox {
    id: control
    font.family: Theme.fontSans
    font.pixelSize: Theme.textBody
    implicitHeight: Theme.fieldHeight

    contentItem: Text {
        leftPadding: 10
        rightPadding: 28
        text: control.displayText
        color: Theme.text.hi
        font: control.font
        verticalAlignment: Text.AlignVCenter
        elide: Text.ElideRight
    }

    indicator: Icon {
        name: "chevron-down"
        size: 14
        color: Theme.text.mid
        x: control.width - width - 9
        y: control.topPadding + (control.availableHeight - height) / 2
    }

    background: Rectangle {
        radius: Theme.radiusSm
        color: control.hovered ? Theme.surface.hover : Theme.surface.raised
        border.color: control.visualFocus ? Theme.accent.brand : Theme.border.base
        border.width: control.visualFocus ? Theme.focusRing : 1
        Behavior on color { ColorAnimation { duration: Theme.motionFast } }
    }

    popup: Popup {
        y: control.height + 4
        width: control.width
        padding: 4
        background: Rectangle {
            color: Theme.surface.panel
            border.color: Theme.border.strong
            radius: Theme.radiusSm

            Rectangle {
                anchors.fill: parent
                anchors.topMargin: Theme.elevationOffset
                radius: parent.radius
                color: Theme.shadowColor
                z: -1
            }
        }
        contentItem: ListView {
            clip: true
            implicitHeight: contentHeight
            model: control.popup.visible ? control.delegateModel : null
            currentIndex: control.highlightedIndex
            ScrollBar.vertical: ScrollBar {}
        }
    }

    delegate: ItemDelegate {
        required property var modelData
        required property int index
        width: control.width - 8
        height: Theme.controlHeight
        contentItem: Text {
            text: modelData
            color: highlighted ? Theme.text.onAccent : Theme.text.hi
            font: control.font
            elide: Text.ElideRight
            verticalAlignment: Text.AlignVCenter
        }
        highlighted: control.highlightedIndex === index
        background: Rectangle {
            radius: Theme.radiusSm - 1
            color: highlighted ? Theme.accent.brand : "transparent"
        }
    }
}
