package org.kmipkit.ide.lsp

import com.google.gson.Gson
import com.google.gson.JsonArray
import com.google.gson.JsonElement
import com.google.gson.JsonObject
import com.google.gson.JsonParser
import com.intellij.openapi.Disposable
import com.intellij.openapi.components.Service
import com.intellij.openapi.project.Project
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.cancel
import kotlinx.coroutines.withContext
import org.kmipkit.ide.binary.KmipBinaryManager
import org.kmipkit.ide.settings.KmipSettingsState
import java.io.BufferedInputStream
import java.io.BufferedOutputStream
import java.io.ByteArrayOutputStream
import java.nio.charset.StandardCharsets
import java.util.concurrent.atomic.AtomicInteger

@Service(Service.Level.PROJECT)
class KmipLspService(private val project: Project) : Disposable {
    val scope: CoroutineScope = CoroutineScope(SupervisorJob() + Dispatchers.IO)
    private val gson = Gson()
    private val nextId = AtomicInteger(1)
    private val rpcLock = Any()

    private var process: Process? = null
    private var reader: BufferedInputStream? = null
    private var writer: BufferedOutputStream? = null

    suspend fun decodeTtlv(input: String, revealOffsets: List<Int> = emptyList()): JsonElement =
        withContext(Dispatchers.IO) {
            val params = JsonObject().apply {
                addProperty("input", input)
                addProperty("format", "auto")
                val arr = JsonArray()
                revealOffsets.forEach { arr.add(it) }
                add("revealOffsets", arr)
            }
            sendRpcRequest("kmip/decodeTtlv", params)
        }

    suspend fun executeRequest(params: JsonObject): JsonElement =
        withContext(Dispatchers.IO) {
            sendRpcRequest("kmip/executeRequest", params)
        }

    suspend fun generateTest(input: String, language: String): JsonElement =
        withContext(Dispatchers.IO) {
            val params = JsonObject().apply {
                addProperty("input", input)
                addProperty("language", language)
            }
            sendRpcRequest("kmip/generateTest", params)
        }

    private fun ensureProcessStarted() {
        val existing = process
        if (existing != null && existing.isAlive) return

        val settings = KmipSettingsState.getInstance().state
        val bin = KmipBinaryManager.resolveBinary(settings.binaryPath, project.basePath)
        val started = ProcessBuilder(bin.absolutePath, "--stdio")
            .redirectError(ProcessBuilder.Redirect.INHERIT)
            .start()

        process = started
        reader = BufferedInputStream(started.inputStream)
        writer = BufferedOutputStream(started.outputStream)

        // Perform LSP initialize handshake
        sendRpcInternal("initialize", JsonObject())
    }

    private fun sendRpcRequest(method: String, params: JsonObject): JsonElement {
        synchronized(rpcLock) {
            ensureProcessStarted()
            return sendRpcInternal(method, params)
        }
    }

    private fun sendRpcInternal(method: String, params: JsonObject): JsonElement {
        val out = writer ?: throw IllegalStateException("kmip-lsp stdin unavailable")
        val input = reader ?: throw IllegalStateException("kmip-lsp stdout unavailable")

        val reqId = nextId.getAndIncrement()
        val payload = JsonObject().apply {
            addProperty("jsonrpc", "2.0")
            addProperty("id", reqId)
            addProperty("method", method)
            add("params", params)
        }

        val jsonBytes = gson.toJson(payload).toByteArray(StandardCharsets.UTF_8)
        val header = "Content-Length: ${jsonBytes.size}\r\n\r\n".toByteArray(StandardCharsets.US_ASCII)
        out.write(header)
        out.write(jsonBytes)
        out.flush()

        while (true) {
            val message = readFramedMessage(input)
                ?: throw IllegalStateException("kmip-lsp closed output stream unexpectedly")
            val obj = JsonParser.parseString(message).asJsonObject
            if (obj.has("id") && obj.get("id").asInt == reqId) {
                if (obj.has("error")) {
                    val errMsg = obj.getAsJsonObject("error").get("message").asString
                    throw IllegalStateException(errMsg)
                }
                return obj.get("result")
            }
        }
    }

    private fun readFramedMessage(input: BufferedInputStream): String? {
        var contentLength = -1
        while (true) {
            val line = readHeaderLine(input) ?: return null
            if (line.isEmpty()) break
            if (line.startsWith("Content-Length:", ignoreCase = true)) {
                contentLength = line.substringAfter(":").trim().toIntOrNull() ?: -1
            }
        }
        if (contentLength <= 0) return null
        val body = ByteArray(contentLength)
        var offset = 0
        while (offset < contentLength) {
            val n = input.read(body, offset, contentLength - offset)
            if (n < 0) return null
            offset += n
        }
        return String(body, StandardCharsets.UTF_8)
    }

    private fun readHeaderLine(input: BufferedInputStream): String? {
        val buf = ByteArrayOutputStream()
        while (true) {
            val b = input.read()
            if (b == -1) {
                return if (buf.size() == 0) null else String(buf.toByteArray(), StandardCharsets.US_ASCII)
            }
            if (b == '\n'.code) {
                val raw = String(buf.toByteArray(), StandardCharsets.US_ASCII)
                return raw.trimEnd('\r')
            }
            buf.write(b)
        }
    }

    override fun dispose() {
        scope.cancel()
        process?.destroy()
        process = null
    }
}
