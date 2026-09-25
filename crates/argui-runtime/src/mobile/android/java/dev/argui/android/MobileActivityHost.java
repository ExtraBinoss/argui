package dev.argui.android;

import android.Manifest;
import android.app.Activity;
import android.content.Intent;
import android.content.pm.PackageManager;
import android.os.Build;

/** JNI entry points that start and update Argui's Android foreground activity. */
public final class MobileActivityHost {
    private static final int NOTIFICATION_PERMISSION_REQUEST = 6107;

    private MobileActivityHost() {}

    /** Requests Android 13+ notification access, returning whether it is already granted. */
    public static boolean prepareNotificationPermission(Activity activity) {
        if (Build.VERSION.SDK_INT < 33
                || activity.checkSelfPermission(Manifest.permission.POST_NOTIFICATIONS)
                        == PackageManager.PERMISSION_GRANTED) {
            return true;
        }
        activity.runOnUiThread(new Runnable() {
            @Override
            public void run() {
                activity.requestPermissions(
                        new String[] {Manifest.permission.POST_NOTIFICATIONS},
                        NOTIFICATION_PERMISSION_REQUEST);
            }
        });
        return false;
    }

    /** Starts the declared foreground service with its initial notification content. */
    public static void start(Activity activity, String title, String message) {
        Intent intent = new Intent(activity, ForegroundTaskService.class)
                .setAction(ForegroundTaskService.ACTION_START)
                .putExtra(ForegroundTaskService.EXTRA_TITLE, title)
                .putExtra(ForegroundTaskService.EXTRA_MESSAGE, message);
        activity.startForegroundService(intent);
    }

    /** Sends a progress update to the existing foreground service. */
    public static void update(Activity activity, int percent, String message) {
        Intent intent = new Intent(activity, ForegroundTaskService.class)
                .setAction(ForegroundTaskService.ACTION_UPDATE)
                .putExtra(ForegroundTaskService.EXTRA_PROGRESS, percent)
                .putExtra(ForegroundTaskService.EXTRA_MESSAGE, message);
        activity.startService(intent);
    }

    /** Stops the service and removes its notification. */
    public static void finish(Activity activity) {
        activity.stopService(new Intent(activity, ForegroundTaskService.class));
    }
}
