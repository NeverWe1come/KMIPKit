import java.security.MessageDigest
import org.gradle.api.GradleException
import org.gradle.api.tasks.compile.JavaCompile
import org.gradle.api.tasks.bundling.Jar

plugins {
    `java-library`
}

group = "org.kmipkit"
version = "1.0.0"
description = "KMIPKit Java 17 bindings, version 1.0.0 (unreleased)"

repositories {
    mavenCentral()
}

tasks.register("apiAuditClasspath") {
    doLast {
        println("KMIPKIT_API_AUDIT_CLASSPATH=" + sourceSets.main.get().compileClasspath.asPath)
    }
}

dependencies {
    api("org.jspecify:jspecify:1.0.0")
    testImplementation("org.junit.jupiter:junit-jupiter:5.10.2")
    testRuntimeOnly("org.junit.platform:junit-platform-launcher")
}

tasks.withType<JavaCompile>().configureEach {
    options.release.set(17)
    options.encoding = "UTF-8"
}

tasks.test {
    useJUnitPlatform()
    dependsOn("buildNativeLibrary")
    systemProperty("kmipkit.nativeLibrary", layout.projectDirectory.file("target/native/${hostLibraryName()}").asFile.absolutePath)
}

fun hostTargetId(): String {
    val os = System.getProperty("os.name").lowercase()
    val arch = System.getProperty("os.arch").lowercase()
    val x86_64 = arch == "amd64" || arch == "x86_64"
    val aarch64 = arch == "aarch64" || arch == "arm64"
    return when {
        os.startsWith("windows") && x86_64 -> "windows-x86_64-msvc"
        os == "linux" && x86_64 -> "linux-glibc-x86_64"
        os == "linux" && aarch64 -> "linux-glibc-aarch64"
        (os.startsWith("mac") || os == "darwin") && x86_64 -> "macos-x86_64"
        (os.startsWith("mac") || os == "darwin") && aarch64 -> "macos-aarch64"
        else -> throw GradleException("This host is not in the KMIPKit Java native target matrix")
    }
}

fun libraryName(target: String): String = when (target) {
    "windows-x86_64-msvc" -> "kmipkit_jni.dll"
    "linux-glibc-x86_64", "linux-glibc-aarch64" -> "libkmipkit_jni.so"
    "macos-x86_64", "macos-aarch64" -> "libkmipkit_jni.dylib"
    else -> throw GradleException("The requested native target is not in the KMIPKit Java matrix")
}

fun hostLibraryName(): String = libraryName(hostTargetId())

val hostTarget = hostTargetId()
val nativeTarget = providers.gradleProperty("nativeTarget").orElse(hostTarget)
val nativeLibrary = providers.gradleProperty("nativeLibrary").orElse(
    layout.projectDirectory.file("target/native/${hostLibraryName()}").asFile.absolutePath)
val nativeResources = layout.buildDirectory.dir("generated/native-resources")

tasks.register<org.gradle.api.tasks.Exec>("buildNativeLibrary") {
    val windows = System.getProperty("os.name").startsWith("Windows", ignoreCase = true)
    workingDir(projectDir)
    if (windows) {
        commandLine(
            "powershell.exe", "-NoLogo", "-NoProfile", "-ExecutionPolicy", "Bypass",
            "-File", file("native/build.ps1").absolutePath)
    } else {
        commandLine("sh", file("native/build.sh").absolutePath)
    }
    outputs.file(layout.projectDirectory.file("target/native/${hostLibraryName()}"))
}

val prepareNativeResources = tasks.register("prepareNativeResources") {
    dependsOn("buildNativeLibrary")
    inputs.property("nativeTarget", nativeTarget)
    if (nativeLibrary.isPresent) {
        inputs.file(nativeLibrary)
    }
    outputs.dir(nativeResources)
    doLast {
        val target = nativeTarget.get()
        if (target != hostTarget) {
            throw GradleException("Native JAR packaging must run on its target OS and architecture")
        }
        val binary = file(nativeLibrary.get())
        if (!binary.isFile) {
            throw GradleException("The configured native library file is unavailable")
        }

        val expectedName = libraryName(target)
        val resourceDirectory = nativeResources.get().asFile.resolve("META-INF/native/$target")
        if (!resourceDirectory.mkdirs() && !resourceDirectory.isDirectory) {
            throw GradleException("Native resources could not be prepared")
        }
        val bundledLibrary = resourceDirectory.resolve(expectedName)
        binary.copyTo(bundledLibrary, overwrite = true)
        val digest = MessageDigest.getInstance("SHA-256")
        bundledLibrary.inputStream().use { input ->
            val buffer = ByteArray(32 * 1024)
            while (true) {
                val count = input.read(buffer)
                if (count < 0) break
                digest.update(buffer, 0, count)
            }
        }
        resourceDirectory.resolve("$expectedName.sha256").writeText(
            digest.digest().joinToString("") { "%02x".format(it) } + "\n",
            Charsets.US_ASCII)
    }
}

tasks.register<Jar>("nativeJar") {
    dependsOn("buildNativeLibrary")
    dependsOn(prepareNativeResources)
    from(sourceSets.main.get().output)
    from(nativeResources)
    archiveClassifier.set(nativeTarget)
}

val nativeJarSmokeClasses = layout.buildDirectory.dir("classes/nativeJarSmoke")
val compileNativeJarSmoke = tasks.register<JavaCompile>("compileNativeJarSmoke") {
    dependsOn("nativeJar")
    source(file("src/test/nativeJarSmoke/java/NativeLibraryPackageSmoke.java"))
    classpath = files(tasks.named<Jar>("nativeJar").flatMap { it.archiveFile })
    destinationDirectory.set(nativeJarSmokeClasses)
    options.release.set(17)
    options.encoding = "UTF-8"
}

val wrongAbiNativeLibrary = layout.buildDirectory.file("native-package-smoke/${hostLibraryName()}")
val compileWrongAbiNativeLibrary = tasks.register<org.gradle.api.tasks.Exec>("compileWrongAbiNativeLibrary") {
    val fixture = file("../../tools/native_package_smoke/wrong_abi.rs")
    val library = wrongAbiNativeLibrary.get().asFile
    inputs.file(fixture)
    outputs.file(library)
    doFirst {
        library.parentFile.mkdirs()
        commandLine(
            "rustc", "--edition=2024", "--crate-name", "kmipkit_wrong_abi", "--crate-type", "cdylib",
            fixture.absolutePath, "-o", library.absolutePath)
    }
}

tasks.register<org.gradle.api.tasks.JavaExec>("nativeJarSmoke") {
    dependsOn("nativeJar", compileNativeJarSmoke, compileWrongAbiNativeLibrary)
    classpath = files(tasks.named<Jar>("nativeJar").flatMap { it.archiveFile }, nativeJarSmokeClasses)
    mainClass.set("NativeLibraryPackageSmoke")
    args(
        tasks.named<Jar>("nativeJar").flatMap { it.archiveFile }.get().asFile.absolutePath,
        wrongAbiNativeLibrary.get().asFile.absolutePath)
    jvmArgs("-Djava.security.manager=allow")
}
