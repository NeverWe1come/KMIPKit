package org.kmipkit.ide.ui

import com.google.gson.Gson
import com.google.gson.JsonObject
import com.google.gson.JsonParser
import com.intellij.openapi.Disposable
import com.intellij.openapi.application.ApplicationManager
import com.intellij.openapi.components.Service
import com.intellij.openapi.components.service
import com.intellij.openapi.project.Project
import com.intellij.openapi.util.Disposer
import com.intellij.ui.jcef.JBCefBrowser
import com.intellij.ui.jcef.JBCefBrowserBase
import com.intellij.ui.jcef.JBCefJSQuery
import kotlinx.coroutines.launch
import org.cef.browser.CefBrowser
import org.cef.browser.CefFrame
import org.cef.handler.CefLoadHandlerAdapter
import org.kmipkit.ide.lsp.KmipLspService
import javax.swing.JComponent

@Service(Service.Level.PROJECT)
class JcefWebviewBridge(private val project: Project) : Disposable {
    private val gson = Gson()
    private val browser = JBCefBrowser()
    private val jsQuery = JBCefJSQuery.create(browser as JBCefBrowserBase)
    private var isReady = false
    private var pendingInput: String? = null

    val component: JComponent
        get() = browser.component

    init {
        Disposer.register(this, browser)
        Disposer.register(this, jsQuery)

        jsQuery.addHandler { rawJson ->
            handleMessageFromWebview(rawJson)
            null
        }

        browser.jbCefClient.addLoadHandler(object : CefLoadHandlerAdapter() {
            override fun onLoadEnd(cefBrowser: CefBrowser, frame: CefFrame, httpStatusCode: Int) {
                if (frame.isMain) {
                    val inject = """
                        window.__kmipJcefSend = function(jsonPayload) {
                            ${jsQuery.inject("jsonPayload")}
                        };
                    """.trimIndent()
                    cefBrowser.executeJavaScript(inject, cefBrowser.url, 0)
                }
            }
        }, browser.cefBrowser)

        val html = loadBundledWebviewHtml()
        browser.loadHTML(html, "https://kmipkit.local/webview/index.html")
    }

    fun decodeAndDisplay(input: String, revealOffsets: List<Int> = emptyList()) {
        if (!isReady) {
            pendingInput = input
            return
        }
        val lsp = project.service<KmipLspService>()
        lsp.scope.launch {
            try {
                val result = lsp.decodeTtlv(input, revealOffsets)
                val payload = JsonObject().apply {
                    add("result", result)
                    addProperty("sourceInput", input)
                }
                sendToWebview("decodeResult", payload)
            } catch (ex: Exception) {
                val err = JsonObject().apply {
                    addProperty("message", ex.message ?: ex.toString())
                }
                sendToWebview("decodeError", err)
            }
        }
    }

    private fun handleMessageFromWebview(rawJson: String) {
        val obj = runCatching { JsonParser.parseString(rawJson).asJsonObject }.getOrNull() ?: return
        val type = obj.get("type")?.asString ?: return
        when (type) {
            "ready" -> {
                isReady = true
                val pending = pendingInput
                pendingInput = null
                if (!pending.isNullOrBlank()) {
                    decodeAndDisplay(pending)
                }
            }
            "decodeTtlv" -> {
                val payload = obj.getAsJsonObject("payload") ?: return
                val input = payload.get("input")?.asString ?: return
                val offsets = payload.getAsJsonArray("revealOffsets")?.map { it.asInt } ?: emptyList()
                decodeAndDisplay(input, offsets)
            }
            "executeRequest" -> {
                val payload = obj.getAsJsonObject("payload") ?: return
                val lsp = project.service<KmipLspService>()
                lsp.scope.launch {
                    try {
                        val res = lsp.executeRequest(payload)
                        sendToWebview("executeResult", res.asJsonObject)
                    } catch (ex: Exception) {
                        val err = JsonObject().apply { addProperty("message", ex.message ?: ex.toString()) }
                        sendToWebview("executeError", err)
                    }
                }
            }
            "generateTest" -> {
                val payload = obj.getAsJsonObject("payload") ?: return
                val input = payload.get("input")?.asString ?: return
                val language = payload.get("language")?.asString ?: "rust"
                val lsp = project.service<KmipLspService>()
                lsp.scope.launch {
                    try {
                        val res = lsp.generateTest(input, language)
                        sendToWebview("generatedTest", res.asJsonObject)
                    } catch (ex: Exception) {
                        val err = JsonObject().apply { addProperty("message", ex.message ?: ex.toString()) }
                        sendToWebview("decodeError", err)
                    }
                }
            }
        }
    }

    private fun sendToWebview(type: String, payload: JsonObject) {
        val typeJson = gson.toJson(type)
        val payloadJson = gson.toJson(payload)
        ApplicationManager.getApplication().invokeLater {
            val cef = browser.cefBrowser
            cef.executeJavaScript(
                "if (window.__kmipReceiveMessage) { window.__kmipReceiveMessage($typeJson, $payloadJson); }",
                cef.url,
                0
            )
        }
    }

    private fun loadBundledWebviewHtml(): String {
        val css = javaClass.getResourceAsStream("/webview/assets/app.css")?.bufferedReader()?.readText() ?: ""
        val js = javaClass.getResourceAsStream("/webview/assets/app.js")?.bufferedReader()?.readText() ?: ""
        if (js.isBlank()) {
            return "<html><body><p>webview-ui bundle missing. Run <code>ide-plugins/scripts/build-webview.sh</code>.</p></body></html>"
        }
        return """
            <!DOCTYPE html>
            <html lang="en">
            <head>
              <meta charset="UTF-8" />
              <style>$css</style>
            </head>
            <body>
              <div id="app"></div>
              <script type="module">$js</script>
            </body>
            </html>
        """.trimIndent()
    }

    override fun dispose() {}
}
