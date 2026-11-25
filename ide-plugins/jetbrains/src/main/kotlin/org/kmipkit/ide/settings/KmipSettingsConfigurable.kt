package org.kmipkit.ide.settings

import com.intellij.openapi.options.BoundConfigurable
import com.intellij.openapi.ui.DialogPanel
import com.intellij.ui.dsl.builder.bindSelected
import com.intellij.ui.dsl.builder.bindText
import com.intellij.ui.dsl.builder.panel

class KmipSettingsConfigurable : BoundConfigurable("KMIPKit") {
    private val settings = KmipSettingsState.getInstance().state

    override fun createPanel(): DialogPanel = panel {
        group("KMIP Core Binary (kmip-lsp)") {
            row("Binary path (optional):") {
                textField()
                    .bindText(settings::binaryPath)
                    .comment("Leave empty to auto-resolve from workspace build or local cache.")
            }
        }
        group("Protocol & Security") {
            row("Default KMIP version:") {
                textField().bindText(settings::defaultKmipVersion)
            }
            row {
                checkBox("Redact sensitive key material by default")
                    .bindSelected(settings::redactSensitiveByDefault)
            }
        }
    }
}
