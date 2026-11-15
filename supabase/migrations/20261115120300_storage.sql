-- 4/6 · Storage buckets and policies. Bucket and policy names carry the eva- prefix, and every
-- policy is scoped to an Eva bucket, so they never apply to another app's buckets in a shared project.
--   eva-team-assets  private · reports, slides, images, demo videos, zips (team, MC, crew lead read)
--   eva-public-media public  · images and video for the public site (anyone downloads by URL;
--                              only team / mission control upload)

insert into storage.buckets (id, name, public, file_size_limit, allowed_mime_types)
values
  ('eva-team-assets', 'eva-team-assets', false, 26214400,
   array['image/*', 'video/*', 'audio/*', 'text/*', 'application/pdf', 'application/zip', 'application/json']),
  ('eva-public-media', 'eva-public-media', true, 10485760,
   array['image/png', 'image/jpeg', 'image/webp', 'image/svg+xml', 'image/gif', 'video/mp4', 'video/webm'])
on conflict (id) do nothing;  -- never modify a bucket that already exists

-- ---------------------------------------------------------------- eva-team-assets (private)

drop policy if exists "eva-team-assets read" on storage.objects;
create policy "eva-team-assets read" on storage.objects for select to authenticated
  using (bucket_id = 'eva-team-assets' and (select eva.has_role('team', 'mission_control', 'crew_lead')));

drop policy if exists "eva-team-assets upload" on storage.objects;
create policy "eva-team-assets upload" on storage.objects for insert to authenticated
  with check (bucket_id = 'eva-team-assets'
              and (select eva.has_role('team', 'mission_control'))
              and owner_id = (select auth.uid())::text);

drop policy if exists "eva-team-assets update" on storage.objects;
create policy "eva-team-assets update" on storage.objects for update to authenticated
  using (bucket_id = 'eva-team-assets'
         and (select eva.has_role('team', 'mission_control'))
         and (owner_id = (select auth.uid())::text or (select eva.has_role('mission_control'))))
  with check (bucket_id = 'eva-team-assets');

drop policy if exists "eva-team-assets delete" on storage.objects;
create policy "eva-team-assets delete" on storage.objects for delete to authenticated
  using (bucket_id = 'eva-team-assets'
         and (select eva.has_role('team', 'mission_control'))
         and (owner_id = (select auth.uid())::text or (select eva.has_role('mission_control'))));

-- ---------------------------------------------------------------- eva-public-media (public bucket)
-- Downloads by public URL need no policy. Listing through the API is limited to signed-in roles;
-- writes to team / mission control.

drop policy if exists "eva-public-media list" on storage.objects;
create policy "eva-public-media list" on storage.objects for select to authenticated
  using (bucket_id = 'eva-public-media' and (select eva.my_role()) is not null);

drop policy if exists "eva-public-media upload" on storage.objects;
create policy "eva-public-media upload" on storage.objects for insert to authenticated
  with check (bucket_id = 'eva-public-media'
              and (select eva.has_role('team', 'mission_control'))
              and owner_id = (select auth.uid())::text);

drop policy if exists "eva-public-media update" on storage.objects;
create policy "eva-public-media update" on storage.objects for update to authenticated
  using (bucket_id = 'eva-public-media' and (select eva.has_role('team', 'mission_control')))
  with check (bucket_id = 'eva-public-media');

drop policy if exists "eva-public-media delete" on storage.objects;
create policy "eva-public-media delete" on storage.objects for delete to authenticated
  using (bucket_id = 'eva-public-media' and (select eva.has_role('team', 'mission_control')));
