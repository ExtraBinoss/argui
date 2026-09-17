package dev.argui.android;

import android.app.Notification;
import android.app.NotificationChannel;
import android.app.NotificationManager;
import android.app.PendingIntent;
import android.app.Service;
import android.content.Intent;
import android.content.pm.ServiceInfo;
import android.os.Build;
import android.os.IBinder;

/** Owns an ongoing notification while an Argui task continues outside its activity window. */
public final class ForegroundTaskService extends Service {
    static final String ACTION_START = "dev.argui.android.action.START";
    static final String ACTION_UPDATE = "dev.argui.android.action.UPDATE";
    static final String EXTRA_TITLE = "dev.argui.android.extra.TITLE";
    static final String EXTRA_MESSAGE = "dev.argui.android.extra.MESSAGE";
    static final String EXTRA_PROGRESS = "dev.argui.android.extra.PROGRESS";

    private static final String CHANNEL_ID = "argui_background_activity";
    private static final int NOTIFICATION_ID = 6108;

    private String title = "Argui activity";
    private String message = "Working in the background";
    private int progress;
    private int notificationIcon;

    @Override
    public void onCreate() {
        super.onCreate();
        notificationIcon = getResources().getIdentifier(
                "ic_notification_activity", "drawable", getPackageName());
        if (notificationIcon == 0) {
            throw new IllegalStateException("ic_notification_activity drawable is required");
        }
        createNotificationChannel();
    }

    @Override
    public int onStartCommand(Intent intent, int flags, int startId) {
        if (intent == null) {
            stopSelf(startId);
            return START_NOT_STICKY;
        }
        String action = intent.getAction();
        if (ACTION_START.equals(action)) {
            String nextTitle = intent.getStringExtra(EXTRA_TITLE);
            String nextMessage = intent.getStringExtra(EXTRA_MESSAGE);
            if (nextTitle != null) {
                title = nextTitle;
            }
            if (nextMessage != null) {
                message = nextMessage;
            }
            progress = 0;
        } else if (ACTION_UPDATE.equals(action)) {
            progress = Math.max(0, Math.min(100, intent.getIntExtra(EXTRA_PROGRESS, progress)));
            String nextMessage = intent.getStringExtra(EXTRA_MESSAGE);
            if (nextMessage != null) {
                message = nextMessage;
            }
        }
        startAsForeground();
        return START_NOT_STICKY;
    }

    @Override
    public void onDestroy() {
        stopForeground(STOP_FOREGROUND_REMOVE);
        super.onDestroy();
    }

    /** Stops cleanly when Android expires the foreground-service time allowance. */
    @Override
    public void onTimeout(int startId, int foregroundServiceType) {
        stopForeground(STOP_FOREGROUND_REMOVE);
        stopSelf(startId);
    }

    @Override
    public IBinder onBind(Intent intent) {
        return null;
    }

    /** Creates the silent, low-importance notification channel used by this service. */
    private void createNotificationChannel() {
        if (Build.VERSION.SDK_INT < 26) {
            return;
        }
        NotificationChannel channel = new NotificationChannel(
                CHANNEL_ID, "Background activity", NotificationManager.IMPORTANCE_LOW);
        channel.setDescription("Progress for work started in Argui");
        channel.setSound(null, null);
        channel.enableVibration(false);
        getSystemService(NotificationManager.class).createNotificationChannel(channel);
    }

    /** Publishes the foreground-service notification with the current task progress. */
    private void startAsForeground() {
        Notification notification = buildNotification();
        if (Build.VERSION.SDK_INT >= 29) {
            startForeground(
                    NOTIFICATION_ID,
                    notification,
                    ServiceInfo.FOREGROUND_SERVICE_TYPE_DATA_SYNC);
        } else {
            startForeground(NOTIFICATION_ID, notification);
        }
    }

    /** Builds the ongoing progress notification and its tap-to-return action. */
    private Notification buildNotification() {
        Intent launchIntent = getPackageManager().getLaunchIntentForPackage(getPackageName());
        PendingIntent contentIntent = launchIntent == null
                ? null
                : PendingIntent.getActivity(
                        this,
                        0,
                        launchIntent,
                        PendingIntent.FLAG_UPDATE_CURRENT | PendingIntent.FLAG_IMMUTABLE);
        Notification.Builder builder = new Notification.Builder(this, CHANNEL_ID)
                .setSmallIcon(notificationIcon)
                .setContentTitle(title)
                .setContentText(message)
                .setCategory(Notification.CATEGORY_PROGRESS)
                .setOngoing(true)
                .setOnlyAlertOnce(true)
                .setShowWhen(false)
                .setProgress(100, progress, false);
        if (contentIntent != null) {
            builder.setContentIntent(contentIntent);
        }
        return builder.build();
    }
}
