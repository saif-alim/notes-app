import java.io.ByteArrayOutputStream

plugins {
    alias(libs.plugins.android.application)
    alias(libs.plugins.kotlin.android)
    alias(libs.plugins.kotlin.compose)
}

// Pre-build task: cross-compile platform-core .so via cargo-ndk.
// Requires: cargo-ndk installed (`cargo install cargo-ndk`) + Android NDK.
// Set ANDROID_NDK_HOME env var to the NDK root.
// Skip on CI if SKIP_NATIVE_BUILD=true.
val buildNativeLibs = tasks.register("buildNativeLibs") {
    group = "build"
    description = "Cross-compile platform-core shared library for Android ABIs via cargo-ndk"
    onlyIf { System.getenv("SKIP_NATIVE_BUILD") != "true" }
    doLast {
        val script = rootDir.resolve("../../tools/build-android.sh").canonicalFile
        if (!script.exists()) error("build-android.sh not found at $script")
        exec {
            commandLine("bash", script.absolutePath)
            workingDir = rootDir.resolve("../../..").canonicalFile
            environment("OUT_DIR", layout.projectDirectory.file("src/main/jniLibs").asFile.absolutePath)
        }
    }
}

tasks.named("preBuild") {
    dependsOn(buildNativeLibs)
}

android {
    namespace = "com.notes"
    compileSdk = 35

    defaultConfig {
        applicationId = "com.notes"
        minSdk = 26
        targetSdk = 35
        versionCode = 1
        versionName = "1.0"

        testInstrumentationRunner = "androidx.test.runner.AndroidJUnitRunner"
    }

    buildTypes {
        release {
            isMinifyEnabled = false
        }
    }

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }

    kotlinOptions {
        jvmTarget = "17"
    }

    buildFeatures {
        compose = true
    }

    // JNA uses reflection; keep relevant classes
    packagingOptions {
        resources {
            excludes += "/META-INF/{AL2.0,LGPL2.1}"
        }
    }
}

dependencies {
    implementation(libs.androidx.core.ktx)
    implementation(libs.androidx.lifecycle.runtime.ktx)
    implementation(libs.androidx.lifecycle.viewmodel.compose)
    implementation(libs.androidx.activity.compose)
    implementation(platform(libs.androidx.compose.bom))
    implementation(libs.androidx.ui)
    implementation(libs.androidx.ui.graphics)
    implementation(libs.androidx.ui.tooling.preview)
    implementation(libs.androidx.material3)

    // JNA — required by uniffi-generated Kotlin bindings
    implementation(libs.jna)

    debugImplementation(libs.androidx.ui.tooling)
    debugImplementation(libs.androidx.ui.test.manifest)

    testImplementation(libs.junit)
    androidTestImplementation(libs.androidx.junit)
    androidTestImplementation(libs.androidx.espresso.core)
    androidTestImplementation(platform(libs.androidx.compose.bom))
    androidTestImplementation(libs.androidx.ui.test.junit4)
}
