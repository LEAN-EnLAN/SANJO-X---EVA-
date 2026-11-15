package tech.sanjox.eva;

import android.Manifest;
import android.webkit.PermissionRequest;

import java.util.ArrayList;
import java.util.LinkedHashSet;
import java.util.List;
import java.util.Set;

/**
 * Which WebView media requests the app may grant: microphone and camera only, only to the configured hub origin,
 * and only once the matching Android runtime permission is held. Pure logic (string constants only), unit-tested.
 */
public final class MediaPolicy {
    private MediaPolicy() {}

    /** The subset of {@code resources} Eva ever grants (audio and video capture). */
    public static String[] supported(String[] resources) {
        List<String> out = new ArrayList<>();
        if (resources != null) {
            for (String r : resources) {
                if (PermissionRequest.RESOURCE_AUDIO_CAPTURE.equals(r) || PermissionRequest.RESOURCE_VIDEO_CAPTURE.equals(r)) {
                    if (!out.contains(r)) out.add(r);
                }
            }
        }
        return out.toArray(new String[0]);
    }

    /** Android runtime permissions needed for the supported resources. */
    public static String[] androidPermissionsFor(String[] resources) {
        Set<String> out = new LinkedHashSet<>();
        for (String r : supported(resources)) {
            if (PermissionRequest.RESOURCE_AUDIO_CAPTURE.equals(r)) {
                out.add(Manifest.permission.RECORD_AUDIO);
            } else if (PermissionRequest.RESOURCE_VIDEO_CAPTURE.equals(r)) {
                out.add(Manifest.permission.CAMERA);
            }
        }
        return out.toArray(new String[0]);
    }

    /**
     * Resources to grant now. Empty means deny. {@code sameOrigin} must be the result of
     * {@link HubConfig#isSameOrigin} for the request's origin.
     */
    public static String[] toGrant(String[] resources, boolean sameOrigin, boolean micGranted, boolean cameraGranted) {
        if (!sameOrigin) return new String[0];
        List<String> out = new ArrayList<>();
        for (String r : supported(resources)) {
            if (PermissionRequest.RESOURCE_AUDIO_CAPTURE.equals(r) && micGranted) out.add(r);
            if (PermissionRequest.RESOURCE_VIDEO_CAPTURE.equals(r) && cameraGranted) out.add(r);
        }
        return out.toArray(new String[0]);
    }
}
