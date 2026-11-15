package tech.sanjox.eva;

import android.app.Activity;
import android.content.Intent;
import android.content.pm.PackageInfo;
import android.content.pm.PackageManager;
import android.os.Bundle;
import android.view.View;
import android.view.inputmethod.EditorInfo;
import android.widget.Button;
import android.widget.EditText;
import android.widget.RadioButton;
import android.widget.RadioGroup;
import android.widget.Switch;
import android.widget.TextView;

/**
 * First run and "Hub settings": hub address, http or https, seat, keep-screen-on, and the pinned certificate
 * (with Forget). Saving opens {@link MainActivity}.
 */
public class SetupActivity extends Activity {
    private HubPrefs prefs;
    private EditText address;
    private RadioGroup scheme;
    private RadioGroup seats;
    private TextView voiceNote;
    private TextView formError;
    private Switch keepOn;
    private View pinRow;
    private TextView pinText;

    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        setContentView(R.layout.activity_setup);
        Insets.applySystemBarPadding(findViewById(R.id.root));
        prefs = new HubPrefs(this);

        address = findViewById(R.id.address);
        scheme = findViewById(R.id.scheme);
        seats = findViewById(R.id.seats);
        voiceNote = findViewById(R.id.voice_note);
        formError = findViewById(R.id.form_error);
        keepOn = findViewById(R.id.keep_on);
        pinRow = findViewById(R.id.pin_row);
        pinText = findViewById(R.id.pin_text);

        for (Seat s : Seat.values()) {
            RadioButton rb = (RadioButton) getLayoutInflater().inflate(R.layout.seat_option, seats, false);
            rb.setId(View.generateViewId());
            rb.setTag(s);
            rb.setText(getString(R.string.seat_option, s.title, s.subtitle));
            seats.addView(rb);
        }

        if (savedInstanceState == null) {
            address.setText(prefs.lastAddress());
            scheme.check(HubConfig.HTTP.equals(prefs.lastScheme()) ? R.id.scheme_http : R.id.scheme_https);
            checkSeat(prefs.lastSeat());
            keepOn.setChecked(prefs.keepScreenOn());
        }
        scheme.setOnCheckedChangeListener((g, id) -> refreshNotes());
        refreshNotes();

        address.setOnEditorActionListener((v, actionId, event) -> {
            if (actionId == EditorInfo.IME_ACTION_GO) {
                connect();
                return true;
            }
            return false;
        });
        address.setOnFocusChangeListener((v, has) -> refreshNotes());
        ((Button) findViewById(R.id.connect)).setOnClickListener(v -> connect());
        findViewById(R.id.forget_pin).setOnClickListener(v -> {
            HubConfig c = currentOrNull();
            if (c != null) prefs.forgetPin(c.host);
            refreshNotes();
        });

        try {
            PackageInfo pi = getPackageManager().getPackageInfo(getPackageName(), 0);
            ((TextView) findViewById(R.id.version)).setText(getString(R.string.version_label, pi.versionName));
        } catch (PackageManager.NameNotFoundException ignored) {
            // version label stays empty
        }
    }

    private void checkSeat(Seat seat) {
        for (int i = 0; i < seats.getChildCount(); i++) {
            View v = seats.getChildAt(i);
            if (v.getTag() == seat) seats.check(v.getId());
        }
    }

    private Seat selectedSeat() {
        View v = seats.findViewById(seats.getCheckedRadioButtonId());
        return v != null && v.getTag() instanceof Seat ? (Seat) v.getTag() : Seat.DEFAULT;
    }

    private String selectedScheme() {
        return scheme.getCheckedRadioButtonId() == R.id.scheme_http ? HubConfig.HTTP : HubConfig.HTTPS;
    }

    private HubConfig currentOrNull() {
        try {
            return HubConfig.parse(address.getText().toString(), selectedScheme(), selectedSeat());
        } catch (IllegalArgumentException e) {
            return null;
        }
    }

    private void refreshNotes() {
        boolean https = HubConfig.HTTPS.equals(selectedScheme());
        voiceNote.setText(https ? R.string.voice_note_https : R.string.voice_note_http);
        HubConfig c = currentOrNull();
        String pin = c == null ? null : prefs.pin(c.host);
        if (pin != null) {
            pinRow.setVisibility(View.VISIBLE);
            pinText.setText(getString(R.string.pinned_cert_detail, c.host, Fingerprints.pretty(pin)));
        } else {
            pinRow.setVisibility(View.GONE);
        }
    }

    private void showError(String msg) {
        formError.setText(msg);
        formError.setVisibility(View.VISIBLE);
    }

    private void connect() {
        HubConfig c;
        try {
            c = HubConfig.parse(address.getText().toString(), selectedScheme(), selectedSeat());
        } catch (IllegalArgumentException e) {
            showError(e.getMessage());
            return;
        }
        prefs.save(c);
        prefs.setKeepScreenOn(keepOn.isChecked());
        startActivity(new Intent(this, MainActivity.class)
                .addFlags(Intent.FLAG_ACTIVITY_CLEAR_TOP | Intent.FLAG_ACTIVITY_SINGLE_TOP));
        finish();
    }
}
