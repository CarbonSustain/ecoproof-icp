import type { Principal } from '@dfinity/principal';
import type { ActorMethod } from '@dfinity/agent';
import type { IDL } from '@dfinity/candid';

export interface Challenge {
  'id' : bigint,
  'latitude' : number,
  'title' : string,
  'expiration' : bigint,
  'longitude' : number,
  'picture_url' : string,
  'radius_m' : number,
}
export type PostStatus = { 'OPEN' : null } |
  { 'PAID' : null } |
  { 'EXPIRED' : null } |
  { 'PENDING' : null };
export type Role = { 'User' : null } |
  { 'Admin' : null } |
  { 'Moderator' : null };
export interface UserSubmission {
  'status' : PostStatus,
  'data_id' : bigint,
  'data' : {
    'latitude' : number,
    'submission_photo_url' : string,
    'temperature' : number,
    'city' : string,
    'longitude' : number,
    'timestamp' : bigint,
    'weather' : string,
  },
  'user' : string,
  'expiration_timestamp' : bigint,
  'rewarded' : boolean,
}
export interface UserSubmissionSummary {
  'status' : PostStatus,
  'data_id' : bigint,
  'city' : string,
}
export interface Vote {
  'data_id' : bigint,
  'user' : string,
  'vote_value' : boolean,
  'submission_id' : bigint,
}
export interface VoteSummary {
  'upvotes' : number,
  'data_id' : bigint,
  'downvotes' : number,
}
export interface _SERVICE {
  'create_challenge' : ActorMethod<
    [string, number, number, number, bigint, string],
    bigint
  >,
  'create_tg_user' : ActorMethod<
    [string, string, string, string, string, boolean, string],
    string
  >,
  'delete_vote' : ActorMethod<[string, bigint], string>,
  'finalize_post_status' : ActorMethod<[bigint], string>,
  'get_active_challenges' : ActorMethod<[number, number], Array<Challenge>>,
  'get_all_expiration_times' : ActorMethod<
    [],
    Array<{ 'data_id' : bigint, 'expiration_timestamp' : bigint }>
  >,
  'get_all_submissions' : ActorMethod<
    [],
    Array<
      {
        'status' : PostStatus,
        'data_id' : bigint,
        'data' : {
          'latitude' : number,
          'submission_photo_url' : string,
          'temperature' : number,
          'city' : string,
          'longitude' : number,
          'timestamp' : bigint,
          'weather' : string,
        },
        'user' : string,
        'rewarded' : boolean,
      }
    >
  >,
  'get_all_users' : ActorMethod<
    [],
    Array<
      {
        'username' : [] | [string],
        'balance' : bigint,
        'language_code' : [] | [string],
        'role' : Role,
        'wallet_address' : [] | [string],
        'profile_picture_url' : [] | [string],
        'user_id' : string,
        'is_bot' : boolean,
        'first_name' : [] | [string],
        'last_name' : [] | [string],
      }
    >
  >,
  'get_balance' : ActorMethod<[string], bigint>,
  'get_challenge' : ActorMethod<
    [bigint],
    { 'Ok' : Challenge } |
      { 'Err' : string }
  >,
  'get_challenges_by_radius' : ActorMethod<
    [number, number, number],
    Array<Challenge>
  >,
  'get_challenges_expiring_soon' : ActorMethod<[bigint], Array<Challenge>>,
  'get_expiration_time' : ActorMethod<
    [bigint],
    { 'Ok' : bigint } |
      { 'Err' : string }
  >,
  'get_leaderboard_by_total_votes' : ActorMethod<[], Array<VoteSummary>>,
  'get_leaderboard_by_upvotes' : ActorMethod<[], Array<VoteSummary>>,
  'get_post_status' : ActorMethod<[bigint], string>,
  'get_rewarded_submissions' : ActorMethod<[string], Array<UserSubmission>>,
  'get_submission' : ActorMethod<
    [bigint],
    {
        'Ok' : {
          'status' : PostStatus,
          'data_id' : bigint,
          'data' : {
            'latitude' : number,
            'submission_photo_url' : string,
            'temperature' : number,
            'city' : string,
            'longitude' : number,
            'timestamp' : bigint,
            'weather' : string,
          },
          'user' : string,
          'rewarded' : boolean,
        }
      } |
      { 'Err' : string }
  >,
  'get_submissions_by_challenge' : ActorMethod<[bigint], Array<UserSubmission>>,
  'get_submissions_by_city' : ActorMethod<[string], Array<UserSubmission>>,
  'get_tg_user' : ActorMethod<
    [string],
    {
        'Ok' : {
          'username' : [] | [string],
          'balance' : bigint,
          'language_code' : [] | [string],
          'role' : Role,
          'wallet_address' : [] | [string],
          'profile_picture_url' : [] | [string],
          'user_id' : string,
          'is_bot' : boolean,
          'first_name' : [] | [string],
          'last_name' : [] | [string],
        }
      } |
      { 'Err' : string }
  >,
  'get_user_posts' : ActorMethod<
    [string],
    Array<
      {
        'status' : PostStatus,
        'data_id' : bigint,
        'data' : {
          'latitude' : number,
          'submission_photo_url' : string,
          'temperature' : number,
          'city' : string,
          'longitude' : number,
          'timestamp' : bigint,
          'weather' : string,
        },
        'user' : string,
        'rewarded' : boolean,
      }
    >
  >,
  'get_user_role' : ActorMethod<[string], { 'Ok' : Role } | { 'Err' : string }>,
  'get_user_submission_locations' : ActorMethod<
    [string],
    Array<
      {
        'status' : { 'OPEN' : null } |
          { 'PAID' : null } |
          { 'EXPIRED' : null } |
          { 'PENDING' : null },
        'latitude' : number,
        'data_id' : bigint,
        'longitude' : number,
      }
    >
  >,
  'get_user_submission_summary' : ActorMethod<
    [string],
    Array<UserSubmissionSummary>
  >,
  'get_user_submissions_by_challenge' : ActorMethod<
    [string, bigint],
    Array<UserSubmission>
  >,
  'get_vote_summary' : ActorMethod<
    [bigint],
    { 'upvotes' : number, 'data_id' : bigint, 'downvotes' : number }
  >,
  'get_votes_by_user' : ActorMethod<[string], Array<Vote>>,
  'mark_submission_rewarded' : ActorMethod<
    [bigint],
    { 'Ok' : string } |
      { 'Err' : string }
  >,
  'reward_user' : ActorMethod<[bigint], { 'Ok' : string } | { 'Err' : string }>,
  'submit_weather_data' : ActorMethod<
    [string, number, number, string, number, string, string],
    bigint
  >,
  'submit_weather_data_with_challenge' : ActorMethod<
    [string, number, number, string, number, string, string, bigint],
    { 'Ok' : bigint } |
      { 'Err' : string }
  >,
  'update_user_role' : ActorMethod<
    [string, string, Role],
    { 'Ok' : string } |
      { 'Err' : string }
  >,
  'update_vote' : ActorMethod<[string, bigint, boolean], string>,
  'update_wallet_address' : ActorMethod<[string, string], string>,
  'vote_on_data' : ActorMethod<[string, bigint, boolean], string>,
}
export declare const idlFactory: IDL.InterfaceFactory;
export declare const init: (args: { IDL: typeof IDL }) => IDL.Type[];
