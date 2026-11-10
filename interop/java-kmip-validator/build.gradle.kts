import org.gradle.api.GradleException

plugins {
    java
    application
}

fun javaNativeClassifier(): String {
    val os = System.getProperty("os.name").lowercase()
    val archName = System.getProperty("os.arch").lowercase()
    val arch = if (archName == "amd64" || archName == "x86_64") "x86_64"
        else if (archName == "aarch64" || archName == "arm64") "aarch64" else ""
    return when {
        os.startsWith("windows") && arch == "x86_64" -> "windows-x86_64-msvc"
        os == "linux" && arch.isNotEmpty() -> "linux-glibc-$arch"
        (os.startsWith("mac") || os == "darwin") && arch.isNotEmpty() -> "macos-$arch"
        else -> throw GradleException("This host is outside the KMIPKit Java native target matrix")
    }
}

val kmipkitMavenRepo = providers.gradleProperty("kmipkitMavenRepo")
    .orElse(layout.projectDirectory.dir("build/maven-repository").asFile.absolutePath)
val kmipkitClassifier = providers.gradleProperty("kmipkitClassifier").orElse(javaNativeClassifier())

repositories {
    maven { url = uri(kmipkitMavenRepo.get()) }
    mavenCentral()
}

dependencies {
    implementation("org.kmipkit:kmipkit-java:1.0.0:${kmipkitClassifier.get()}")
    implementation("com.fasterxml.jackson.core:jackson-databind:2.21.8")
    testImplementation("org.junit.jupiter:junit-jupiter:5.10.2")
    testRuntimeOnly("org.junit.platform:junit-platform-launcher")
}

java {
    toolchain {
        languageVersion.set(JavaLanguageVersion.of(17))
    }
}

application {
    mainClass.set("org.kmipkit.validator.Main")
}

tasks.test {
    useJUnitPlatform()
}
