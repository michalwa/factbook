% Defines predicates available in the embedded Prolog runtime
% Required in order to build the initial Prolog state

% `@T` represents a tag `T`
:- op(50, fx, @).


% view_entry(Context, View, Entry)
%   where `Context` is of the native blob type `ViewContext`
%
% Free variable serves as a catch-all wildcard view (e.g. `_`) and prevents
% further clauses from matching uninstantiated views.
view_entry(_, V,        _) :- var(V), !.
view_entry(_, { G },    _) :- G.
view_entry(C, @T,       E) :- entry_tag(C, E, T).
view_entry(C, (X, Y),   E) :- view_entry(C, X, E), view_entry(C, Y, E).
view_entry(C, (X; Y),   E) :- view_entry(C, X, E); view_entry(C, Y, E).
view_entry(C, E1: V,    _) :- view_entry(C, V, E1).
view_entry(C, order(X), E) :- set_entry_order_key(C, E, X).


% human_datetime(TimeOrKeyword, T)
% human_datetime(TimeOrKeyword, T0, T)
%
% Like `human_time/2` and `human_time/3` but unifies the last argument with a
% `date/9` compound term instead of a timestamp.
human_datetime(K, D)     :- human_time(K, T),     stamp_date_time(T, D, local).
human_datetime(K, T0, D) :- human_time(K, T0, T), stamp_date_time(T, D, local).

% human_time(TimeOrKeyword, T)
% human_time(TimeOrKeyword, T0, T)
%
% Maps `TimeOrKeyword` to a timestamp `T`, where `TimeOrKeyword` can be an
% absolute time like `Y-M-D`, or a keyword like `tomorrow`, interpreted relative
% to `T0`. `human_time/2` defaults to the current time as `T0`.
human_time(K, T) :- get_time(T0), human_time(K, T0, T).
human_time(Y-M-D, T0, T) :- human_time_([Y, M, D, 0, 0, 0], T0, T).
human_time(K, T0, T) :-
  (atom(K); string(K)),
  atom_codes(K, Cs),
  phrase(human_time_spec(Ts), Cs),
  human_time_(Ts, T0, T).

human_time_(Ts, T0, T) :-
  stamp_date_time(T0, D0, local),
  human_time_date_(Ts, D0, D),
  date_time_stamp(D, T).
human_time_date_(Ts0, D0, D) :-
  D0 =.. [date|Parts0],
  % Xs0 = [Y, M, D, H, Mn, S]; ignore [Off, Tz, Dst], assume local time
  append(Xs0, [_, _, _], Parts0),
  same_length(Xs0, Ts), % Ensure same length for `maplist`
  append(Ts0, _, Ts),
  maplist(human_time_value_, Ts, Xs0, Xs),
  append(Xs, [_, _, _], Parts),
  D =.. [date|Parts].
human_time_value_(V, X, X)   :- var(V), !.
human_time_value_(N, _, N)   :- integer(N).
human_time_value_(+N, X0, X) :- X is X0 + N.
human_time_value_(-N, X0, X) :- X is X0 - N.

human_time_spec([Y, M, D])             --> human_time_spec_([Y, M, D|_]), ws, "same", ws, "time".
human_time_spec([Y, M, D])             --> "same", ws, "time", ws, human_time_spec_([Y, M, D|_]).
human_time_spec(S)                     --> human_time_spec_(S).

human_time_spec_([Y, M, D])            --> integer(Y), "-", integer(M), "-", integer(D).

human_time_spec_([])                   --> "now".
human_time_spec_([_, _, + 1, 0, 0, 0]) --> "tomorrow".
human_time_spec_([_, _, - 1, 0, 0, 0]) --> "yesterday".

human_time_spec_([+ N, _, _, 0, 0, 0]) --> "in", ws, integer(N), ws, "years".
human_time_spec_([+ N, _, _, 0, 0, 0]) --> integer(N), ws, "years", ws, "from", ws, "now".
human_time_spec_([_, + N, _, 0, 0, 0]) --> "in", ws, integer(N), ws, "months".
human_time_spec_([_, + N, _, 0, 0, 0]) --> integer(N), ws, "months", ws, "from", ws, "now".
human_time_spec_([_, _, + N, 0, 0, 0]) --> "in", ws, integer(N), ws, "days".
human_time_spec_([_, _, + N, 0, 0, 0]) --> "in", ws, integer(W), ws, "weeks", { N is W * 7 }.

human_time_spec_([_, _, + N, 0, 0, 0]) --> integer(W), ws, "weeks", ws, "from", ws, "now", { N is W * 7 }.
human_time_spec_([_, _, + N, 0, 0, 0]) --> integer(N), ws, "days", ws, "from", ws, "now".
human_time_spec_([- N, _, _, 0, 0, 0]) --> integer(N), ws, "years", ws, "ago".
human_time_spec_([_, - N, _, 0, 0, 0]) --> integer(N), ws, "months", ws, "ago".
human_time_spec_([_, _, - N, 0, 0, 0]) --> integer(N), ws, "days", ws, "ago".
human_time_spec_([_, _, - N, 0, 0, 0]) --> integer(W), ws, "weeks", ws, "ago", { N is W * 7 }.

human_time_spec_([+ 1, 1, 1, 0, 0, 0]) --> "next", ws, "year".
human_time_spec_([_, + 1, 1, 0, 0, 0]) --> "next", ws, "month".
human_time_spec_([_, _, + 1, 0, 0, 0]) --> "next", ws, "day".

human_time_spec_([- 1, 1, 1, 0, 0, 0]) --> "last", ws, "year".
human_time_spec_([_, - 1, 1, 0, 0, 0]) --> "last", ws, "month".
human_time_spec_([_, _, - 1, 0, 0, 0]) --> "previous", ws, "day".

integer(I) --> digit(D), digits(Ds), { number_codes(I, [D|Ds]) }.

digits([D|Ds]) --> digit(D), !, digits(Ds).
digits([])     --> [].
digit(C)       --> [C], { code_type(C, digit) }.

ws    --> space, ws_.
ws_   --> space, !, ws_.
ws_   --> [].
space --> [C], { code_type(C, space) }.


:- begin_tests(prelude).

test(human_datetime_date, nondet) :-
  % Test both daylight savings cases
  human_datetime(2022-01-02, date(2022, 1, 2, 0, 0, 0.0, _, _, _)),
  human_datetime(2022-06-02, date(2022, 6, 2, 0, 0, 0.0, _, _, _)).

test(human_time_now_atom, nondet)   :- get_time(T), human_time(now, T, T).
test(human_time_now_string, nondet) :- get_time(T), human_time("now", T, T).

test(human_datetime_tomorrow, nondet) :-
  date_time_stamp(date(2020, 1, 2, 13, 14, 15.0, _, _, _), T0),
  human_datetime(tomorrow, T0, date(2020, 1, 3, 0, 0, 0.0, _, _, _)).

test(human_datetime_tomorrow_same_time, nondet) :-
  date_time_stamp(date(2020, 1, 2, 13, 14, 15.0, _, _, _), T0),
  human_datetime("tomorrow same time", T0, date(2020, 1, 3, 13, 14, 15.0, _, _, _)),
  human_datetime("same time tomorrow", T0, date(2020, 1, 3, 13, 14, 15.0, _, _, _)).

test(human_datetime_yesterday, nondet) :-
  date_time_stamp(date(2020, 1, 2, 13, 14, 15.0, _, _, _), T0),
  human_datetime(yesterday, T0, date(2020, 1, 1, 0, 0, 0.0, _, _, _)).

test(human_datetime_yesterday_same_time, nondet) :-
  date_time_stamp(date(2020, 1, 2, 13, 14, 15.0, _, _, _), T0),
  human_datetime("yesterday same time", T0, date(2020, 1, 1, 13, 14, 15.0, _, _, _)).

test(human_datetime_in_5_years, nondet) :-
  date_time_stamp(date(2020, 1, 2, 13, 14, 15.0, _, _, _), T0),
  human_datetime("in 5 years", T0, date(2025, 1, 2, 0, 0, 0.0, _, _, _)),
  human_datetime("5 years from now", T0, date(2025, 1, 2, 0, 0, 0.0, _, _, _)),
  human_datetime("same time 5 years from now", T0, date(2025, 1, 2, 13, 14, 15.0, _, _, _)).

test(human_datetime_in_5_months, nondet) :-
  date_time_stamp(date(2020, 1, 2, 13, 14, 15.0, _, _, _), T0),
  human_datetime("in 5 months", T0, date(2020, 6, 2, 0, 0, 0.0, _, _, _)).

test(human_datetime_in_5_days, nondet) :-
  date_time_stamp(date(2020, 1, 2, 13, 14, 15.0, _, _, _), T0),
  human_datetime("in 5 days", T0, date(2020, 1, 7, 0, 0, 0.0, _, _, _)).

test(human_datetime_in_2_weeks, nondet) :-
  date_time_stamp(date(2020, 1, 2, 13, 14, 15.0, _, _, _), T0),
  human_datetime("in 2 weeks", T0, date(2020, 1, 16, 0, 0, 0.0, _, _, _)).

test(human_datetime_5_years_ago, nondet) :-
  date_time_stamp(date(2020, 1, 2, 13, 14, 15.0, _, _, _), T0),
  human_datetime("5 years ago", T0, date(2015, 1, 2, 0, 0, 0.0, _, _, _)),
  human_datetime("same time 5 years ago", T0, date(2015, 1, 2, 13, 14, 15.0, _, _, _)).

test(human_datetime_5_months_ago, nondet) :-
  date_time_stamp(date(2020, 1, 2, 13, 14, 15.0, _, _, _), T0),
  human_datetime("5 months ago", T0, date(2019, 8, 2, 0, 0, 0.0, _, _, _)).

test(human_datetime_5_days_ago, nondet) :-
  date_time_stamp(date(2020, 1, 2, 13, 14, 15.0, _, _, _), T0),
  human_datetime("5 days ago", T0, date(2019, 12, 28, 0, 0, 0.0, _, _, _)).

test(human_datetime_2_weeks_ago, nondet) :-
  date_time_stamp(date(2020, 1, 2, 13, 14, 15.0, _, _, _), T0),
  human_datetime("2 weeks ago", T0, date(2019, 12, 19, 0, 0, 0.0, _, _, _)).

test(human_datetime_next_year, nondet) :-
  date_time_stamp(date(2020, 1, 2, 13, 14, 15.0, _, _, _), T0),
  human_datetime("next year", T0, date(2021, 1, 1, 0, 0, 0.0, _, _, _)).

test(human_datetime_next_month, nondet) :-
  date_time_stamp(date(2020, 1, 2, 13, 14, 15.0, _, _, _), T0),
  human_datetime("next month", T0, date(2020, 2, 1, 0, 0, 0.0, _, _, _)).

test(human_datetime_next_day, nondet) :-
  date_time_stamp(date(2020, 1, 2, 13, 14, 15.0, _, _, _), T0),
  human_datetime("next day", T0, date(2020, 1, 3, 0, 0, 0.0, _, _, _)).

test(human_datetime_last_year, nondet) :-
  date_time_stamp(date(2020, 1, 2, 13, 14, 15.0, _, _, _), T0),
  human_datetime("last year", T0, date(2019, 1, 1, 0, 0, 0.0, _, _, _)).

test(human_datetime_last_month, nondet) :-
  date_time_stamp(date(2020, 1, 2, 13, 14, 15.0, _, _, _), T0),
  human_datetime("last month", T0, date(2019, 12, 1, 0, 0, 0.0, _, _, _)).

test(human_datetime_previous_day, nondet) :-
  date_time_stamp(date(2020, 1, 2, 13, 14, 15.0, _, _, _), T0),
  human_datetime("previous day", T0, date(2020, 1, 1, 0, 0, 0.0, _, _, _)).

:- end_tests(prelude).
