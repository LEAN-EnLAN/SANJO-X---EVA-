-- 4/6 · Storage buckets and policies.
--   team-assets  private · reports, slides, images, demo videos, zips (team, MC, crew lead read)
--   public-media public  · images and video for the public site (anyone downloads by URL;
--                          only team / mission control upload)

insert into storage.buckets (id, name, public, file_size_limit, allowed_mime_types)
values
  ('team-assets', 'team-assets', false, 26214400,
   array['image/*', 'video/*', 'audio/*', 'text/*', 'application/pdf', 'application/zip', 'application/json']),
  ('public-media', 'public-media', true, 10485760,
   array['image/png', 'image/jpeg', 'image/webp', 'image/svg+xml', 'image/gif', 'video/mp4', 'video/webm'])
on conflict (id) do update
  set public = excluded.public,
      file_size_limit = excluded.file_size_limit,
      allowed_mime_types = excluded.allowed_mime_types;

-- ---------------------------------------------------------------- team-assets (private)

drop policy if exists "team-assets read" on storage.objects;
create policy "team-assets read" on storage.objects for select to authenticated
  using (bucket_id = 'team-assets' and (select public.has_role('team', 'mission_control', 'crew_lead')));

drop policy if exists "team-assets upload" on storage.objects;
create policy "team-assets upload" on storage.objects for insert to authenticated
  with check (bucket_id = 'team-assets'
              and (select public.has_role('team', 'mission_control'))
              and owner_id = (select auth.uid())::text);

drop policy if exists "team-assets update" on storage.objects;
create policy "team-assets update" on storage.objects for update to authenticated
  using (bucket_id = 'team-assets'
         and (select public.has_role('team', 'mission_control'))
         and (owner_id = (select auth.uid())::text or (select public.has_role('mission_control'))))
  with check (bucket_id = 'team-assets');

drop policy if exists "team-assets delete" on storage.objects;
create policy "team-assets delete" on storage.objects for delete to authenticated
  using (bucket_id = 'team-assets'
         and (select public.has_role('team', 'mission_control'))
         and (owner_id = (select auth.uid())::text or (select public.has_role('mission_control'))));

-- ---------------------------------------------------------------- public-media (public bucket)
-- Downloads by public URL need no policy. Listing through the API is limited to signed-in roles;
-- writes to team / mission control.

drop policy if exists "public-media list" on storage.objects;
create policy "public-media list" on storage.objects for select to authenticated
  using (bucket_id = 'public-media' and (select public.my_role()) is not null);

drop policy if exists "public-media upload" on storage.objects;
create policy "public-media upload" on storage.objects for insert to authenticated
  with check (bucket_id = 'public-media'
              and (select public.has_role('team', 'mission_control'))
              and owner_id = (select auth.uid())::text);

drop policy if exists "public-media update" on storage.objects;
create policy "public-media update" on storage.objects for update to authenticated
  using (bucket_id = 'public-media' and (select public.has_role('team', 'mission_control')))
  with check (bucket_id = 'public-media');

drop policy if exists "public-media delete" on storage.objects;
create policy "public-media delete" on storage.objects for delete to authenticated
  using (bucket_id = 'public-media' and (select public.has_role('team', 'mission_control')));
