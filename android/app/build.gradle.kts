import java.nio.file.Files
import java.util.Properties

plugins {
    id("com.android.application")
}

// Release signing: the keystore lives OUTSIDE the repo. Point EVA_SIGNING_PROPS (env) or
// -PevaSigningProps=<file> at a properties file with storeFile, storePassword, keyAlias, keyPassword.
// Without it, assembleRelease still builds, unsigned.
val signingPropsPath: String? =
    (project.findProperty("evaSigningProps") as String?) ?: System.getenv("EVA_SIGNING_PROPS")
val signingProps: Properties? = signingPropsPath?.let { path ->
    val f = file(path)
    if (f.isFile) Properties().apply { f.inputStream().use { load(it) } } else null
}

android {
    namespace = "tech.sanjox.eva"
    compileSdk = 36

    defaultConfig {
        applicationId = "tech.sanjox.eva"
        minSdk = 26
        targetSdk = 36
        versionCode = 302
        versionName = "0.3.2"
    }

    signingConfigs {
        if (signingProps != null) {
            create("release") {
                storeFile = file(signingProps.getProperty("storeFile"))
                storePassword = signingProps.getProperty("storePassword")
                keyAlias = signingProps.getProperty("keyAlias")
                keyPassword = signingProps.getProperty("keyPassword")
            }
        }
    }

    buildTypes {
        getByName("debug") {
            applicationIdSuffix = ".debug"
            versionNameSuffix = "-debug"
        }
        getByName("release") {
            isMinifyEnabled = true
            isShrinkResources = true
            proguardFiles(getDefaultProguardFile("proguard-android-optimize.txt"), "proguard-rules.pro")
            if (signingProps != null) signingConfig = signingConfigs.getByName("release")
        }
    }

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }

    lint {
        abortOnError = true
        warningsAsErrors = false
        checkReleaseBuilds = true
        htmlReport = true
        xmlReport = false
    }

    testOptions {
        unitTests.isReturnDefaultValues = true
        unitTests.isIncludeAndroidResources = true // Robolectric loads the real layouts and strings
    }
}

base {
    archivesName.set("eva-v0.3.2")
}

// Robolectric runs offline against an android-all jar that Gradle resolves (Robolectric's own downloader
// insists on .sha512 files that Maven Central does not have for these artifacts). The jar is linked, not copied.
val robolectricAndroidAll: Configuration by configurations.creating
val robolectricDepsDir = layout.buildDirectory.dir("robolectric-deps")
val linkRobolectricAndroidAll by tasks.registering {
    inputs.files(robolectricAndroidAll)
    outputs.dir(robolectricDepsDir)
    doLast {
        val dir = robolectricDepsDir.get().asFile.toPath()
        Files.createDirectories(dir)
        robolectricAndroidAll.files.forEach { jar ->
            val link = dir.resolve(jar.name)
            Files.deleteIfExists(link)
            Files.createSymbolicLink(link, jar.toPath())
        }
    }
}
tasks.withType<Test>().configureEach {
    dependsOn(linkRobolectricAndroidAll)
    systemProperty("robolectric.offline", "true")
    systemProperty("robolectric.dependency.dir", robolectricDepsDir.get().asFile.absolutePath)
    maxHeapSize = "1g"
}

dependencies {
    robolectricAndroidAll("org.robolectric:android-all-instrumented:14-robolectric-10818077-i7")
    testImplementation("junit:junit:4.13.2")
    testImplementation("org.robolectric:robolectric:4.16.1")
    testImplementation("org.mockito:mockito-core:5.14.2")
}
