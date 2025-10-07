package org.holochain.androidserviceruntime.plugin.client

import android.app.Activity
import android.content.ComponentName
import android.content.Intent
import android.content.pm.PackageManager
import android.os.Build
import android.util.Log
import android.widget.Button
import androidx.cardview.widget.CardView

class DisconnectedNotice(
    private val activity: Activity,
    private val servicePackage: String,
) : Notice(
        activity,
        R.layout.disconnected_notice,
        "DisconnectedNotice",
    ) {
    companion object {
        private const val TAG = "DisconnectedNotice"
        private const val ASR_SETTINGS_ACTION = "com.android.settings.action.IA_SETTINGS"
    }

    override fun setupNoticeCardView(noticeView: CardView) {
        // Start the app that contains the HolochainService
        noticeView.findViewById<Button>(R.id.openSettingsAction).setOnClickListener {
            openAsr()
        }

        // Restart this app
        noticeView.findViewById<Button>(R.id.restartAction).setOnClickListener {
            super.restartApp()
        }
    }

    private fun openAsr() {
        if (!trySettings()) {
            tryLauncher()
        }
    }

    private fun trySettings(): Boolean =
        try {
            val pm = activity.packageManager
            // Resolve the settings activity dynamically
            val probe = Intent(ASR_SETTINGS_ACTION).setPackage(servicePackage)
            val resolve =
                if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
                    pm
                        .queryIntentActivities(
                            probe,
                            PackageManager.ResolveInfoFlags.of(0),
                        ).firstOrNull()
                } else {
                    @Suppress("DEPRECATION")
                    pm.queryIntentActivities(probe, 0).firstOrNull()
                } ?: return false
            val target =
                ComponentName(resolve.activityInfo.packageName, resolve.activityInfo.name)
            val intent =
                Intent(ASR_SETTINGS_ACTION).apply {
                    component = target
                    addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
                }
            activity.startActivity(intent)
            true
        } catch (t: Throwable) {
            Log.w(TAG, "Settings intent failed", t)
            false
        }

    private fun tryLauncher(): Boolean =
        try {
            val launch = this.activity.packageManager.getLaunchIntentForPackage(servicePackage) ?: return false
            activity.startActivity(launch)
            true
        } catch (t: Throwable) {
            Log.w(TAG, "Launcher intent failed", t)
            false
        }
}
