package org.kmipkit.ide.actions

import com.intellij.openapi.actionSystem.ActionUpdateThread
import com.intellij.openapi.actionSystem.AnAction
import com.intellij.openapi.actionSystem.AnActionEvent
import com.intellij.openapi.actionSystem.CommonDataKeys
import com.intellij.openapi.components.service
import com.intellij.openapi.wm.ToolWindowManager
import org.kmipkit.ide.ui.JcefWebviewBridge

class DecodeTtlvSelectionAction : AnAction() {
    override fun getActionUpdateThread(): ActionUpdateThread = ActionUpdateThread.BGT

    override fun update(e: AnActionEvent) {
        val editor = e.getData(CommonDataKeys.EDITOR)
        e.presentation.isEnabledAndVisible = editor?.selectionModel?.hasSelection() == true
    }

    override fun actionPerformed(e: AnActionEvent) {
        val project = e.project ?: return
        val editor = e.getData(CommonDataKeys.EDITOR) ?: return
        val selected = editor.selectionModel.selectedText?.trim() ?: return
        if (selected.isEmpty()) return

        val toolWindow = ToolWindowManager.getInstance(project).getToolWindow("KMIP")
        toolWindow?.show {
            project.service<JcefWebviewBridge>().decodeAndDisplay(selected)
        }
    }
}
