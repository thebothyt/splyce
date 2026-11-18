plugins {
    alias(libs.plugins.android.application)
}

android {
    namespace = "com.thebothyt.splyce"
    compileSdk = 34

    defaultConfig {
        applicationId = "com.thebothyt.splyce"
        minSdk = 26
        targetSdk = 34
        versionCode = 1
        versionName = "0.1.0"
    }

    buildTypes {
        release {
            isMinifyEnabled = true
            proguardFiles(getDefaultProguardFile("proguard-android-optimize.txt"), "proguard-rules.pro")
        }
    }

    // .so files come from `cargo ndk -o app/src/main/jniLibs`
    sourceSets["main"].jniLibs.srcDir("src/main/jniLibs")
}
