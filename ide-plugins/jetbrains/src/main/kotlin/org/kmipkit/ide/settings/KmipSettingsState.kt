package org.kmipkit.ide.settings

import com.intellij.openapi.application.ApplicationManager
import com.intellij.openapi.components.PersistentStateComponent
import com.intellij.openapi.components.State
import com.intellij.openapi.components.Storage

@State(
    name = "KmipKitSettings",
    storages = [Storage("kmipkit.xml")]
)
class KmipSettingsState : PersistentStateComponent<KmipSettingsState.State> {
    data class State(
        var binaryPath: String = "",
        var defaultKmipVersion: String = "2.1",
        var redactSensitiveByDefault: Boolean = true
    )

    private var state = State()

    override fun getState(): State = state

    override fun loadState(state: State) {
        this.state = state
    }

    companion object {
        fun getInstance(): KmipSettingsState {
            return ApplicationManager.getApplication().getService(KmipSettingsState::class.java)
        }
    }
}
