package tech.sanjox.eva;

import android.content.ContentProvider;
import android.content.ContentValues;
import android.database.Cursor;
import android.database.MatrixCursor;
import android.net.Uri;
import android.os.ParcelFileDescriptor;
import android.provider.OpenableColumns;

import java.io.File;
import java.io.FileNotFoundException;

/**
 * Minimal private provider so the camera app can write a photo or video for an
 * {@code <input type="file" capture>} upload: {@code content://<package>.capture/<name>} maps to
 * {@code cacheDir/capture/<name>}. Not exported; access only through one-off URI grants.
 */
public final class CaptureProvider extends ContentProvider {
    static final String DIR = "capture";

    static String authority(android.content.Context ctx) {
        return ctx.getPackageName() + ".capture";
    }

    static Uri uriFor(android.content.Context ctx, File f) {
        return new Uri.Builder().scheme("content").authority(authority(ctx)).appendPath(f.getName()).build();
    }

    static File dir(android.content.Context ctx) {
        File d = new File(ctx.getCacheDir(), DIR);
        //noinspection ResultOfMethodCallIgnored
        d.mkdirs();
        return d;
    }

    /** Deletes captures older than a day. */
    static void prune(android.content.Context ctx) {
        File[] files = dir(ctx).listFiles();
        if (files == null) return;
        long cutoff = System.currentTimeMillis() - 24L * 3600 * 1000;
        for (File f : files) {
            if (f.lastModified() < cutoff) {
                //noinspection ResultOfMethodCallIgnored
                f.delete();
            }
        }
    }

    private File fileFor(Uri uri) throws FileNotFoundException {
        if (getContext() == null || uri.getPathSegments().size() != 1) throw new FileNotFoundException(uri.toString());
        String name = uri.getLastPathSegment();
        if (name == null || !name.matches("^[A-Za-z0-9_.-]+$") || name.startsWith(".")) {
            throw new FileNotFoundException(uri.toString());
        }
        return new File(dir(getContext()), name);
    }

    @Override
    public boolean onCreate() {
        return true;
    }

    @Override
    public ParcelFileDescriptor openFile(Uri uri, String mode) throws FileNotFoundException {
        return ParcelFileDescriptor.open(fileFor(uri), ParcelFileDescriptor.parseMode(mode));
    }

    @Override
    public String getType(Uri uri) {
        String n = uri.getLastPathSegment() == null ? "" : uri.getLastPathSegment();
        if (n.endsWith(".jpg")) return "image/jpeg";
        if (n.endsWith(".mp4")) return "video/mp4";
        return "application/octet-stream";
    }

    @Override
    public Cursor query(Uri uri, String[] projection, String selection, String[] selectionArgs, String sortOrder) {
        File f;
        try {
            f = fileFor(uri);
        } catch (FileNotFoundException e) {
            return null;
        }
        String[] cols = projection != null ? projection : new String[] {OpenableColumns.DISPLAY_NAME, OpenableColumns.SIZE};
        MatrixCursor c = new MatrixCursor(cols, 1);
        Object[] row = new Object[cols.length];
        for (int i = 0; i < cols.length; i++) {
            if (OpenableColumns.DISPLAY_NAME.equals(cols[i])) row[i] = f.getName();
            else if (OpenableColumns.SIZE.equals(cols[i])) row[i] = f.length();
        }
        c.addRow(row);
        return c;
    }

    @Override
    public Uri insert(Uri uri, ContentValues values) {
        throw new UnsupportedOperationException("read/write through openFile only");
    }

    @Override
    public int delete(Uri uri, String selection, String[] selectionArgs) {
        try {
            return fileFor(uri).delete() ? 1 : 0;
        } catch (FileNotFoundException e) {
            return 0;
        }
    }

    @Override
    public int update(Uri uri, ContentValues values, String selection, String[] selectionArgs) {
        throw new UnsupportedOperationException("read/write through openFile only");
    }
}
