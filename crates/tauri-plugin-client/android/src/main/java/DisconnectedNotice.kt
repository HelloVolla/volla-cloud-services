package org.holochain.androidserviceruntime.plugin.client

import android.app.Activity
import android.content.ActivityNotFoundException
import android.content.Intent
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
        private const val ASR_MAIN = "org.holochain.androidserviceruntime.app.MainActivity"
    }

    override fun setupNoticeCardView(noticeView: CardView) {
        // Start the app that contains the HolochainService
        noticeView.findViewById<Button>(R.id.openSettingsAction).setOnClickListener {
            openAsrInternalSettings()
        }

        // Restart this app
        noticeView.findViewById<Button>(R.id.restartAction).setOnClickListener {
            super.restartApp()
        }
    }

    private fun openAsrInternalSettings(): Boolean =
        try {
            val intent =
                Intent(ASR_SETTINGS_ACTION)
                    .apply {
                        setClassName(servicePackage, ASR_MAIN)
                        addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
                    }

            val resolvable = activity.packageManager.queryIntentActivities(intent, 0).isNotEmpty()
            if (resolvable) {
                activity.startActivity(intent)
                true
            } else {
                Log.w(TAG, "ASR internal settings intent not resolvable")
                false
            }
        } catch (e: ActivityNotFoundException) {
            Log.e(TAG, "ASR IA_SETTINGS activity not found", e)
            false
        } catch (e: SecurityException) {
            Log.e(TAG, "ASR activity not exported or blocked", e)
            false
        } catch (t: Throwable) {
            Log.e(TAG, "Failed to open ASR internal settings", t)
            false
        }
}
