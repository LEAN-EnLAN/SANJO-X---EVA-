package tech.sanjox.eva;

import static org.junit.Assert.assertArrayEquals;

import org.junit.Test;

public class MediaPolicyTest {
    static final String AUDIO = "android.webkit.resource.AUDIO_CAPTURE";
    static final String VIDEO = "android.webkit.resource.VIDEO_CAPTURE";
    static final String DRM = "android.webkit.resource.PROTECTED_MEDIA_ID";
    static final String MIDI = "android.webkit.resource.MIDI_SYSEX";

    @Test
    public void onlyMicAndCameraAreSupported() {
        assertArrayEquals(new String[] {AUDIO, VIDEO}, MediaPolicy.supported(new String[] {AUDIO, DRM, VIDEO, MIDI, AUDIO}));
        assertArrayEquals(new String[0], MediaPolicy.supported(new String[] {DRM, MIDI}));
        assertArrayEquals(new String[0], MediaPolicy.supported(null));
    }

    @Test
    public void androidPermissionsMapping() {
        assertArrayEquals(new String[] {"android.permission.RECORD_AUDIO"}, MediaPolicy.androidPermissionsFor(new String[] {AUDIO}));
        assertArrayEquals(new String[] {"android.permission.RECORD_AUDIO", "android.permission.CAMERA"},
                MediaPolicy.androidPermissionsFor(new String[] {AUDIO, VIDEO, DRM}));
    }

    @Test
    public void grantsOnlyToSameOriginAndHeldPermissions() {
        String[] both = {AUDIO, VIDEO};
        assertArrayEquals(both, MediaPolicy.toGrant(both, true, true, true));
        assertArrayEquals(new String[] {AUDIO}, MediaPolicy.toGrant(both, true, true, false));
        assertArrayEquals(new String[] {VIDEO}, MediaPolicy.toGrant(both, true, false, true));
        assertArrayEquals(new String[0], MediaPolicy.toGrant(both, true, false, false));
        assertArrayEquals("other origin: never", new String[0], MediaPolicy.toGrant(both, false, true, true));
        assertArrayEquals(new String[0], MediaPolicy.toGrant(new String[] {DRM}, true, true, true));
    }
}
