-- Contains global functions that are used by the game

G.FUNCS.has_not_confirmed_matchmaking = function(e)
    if BALATRO_VS_CTX.network.has_confirmed_matchmaking then
        e.config.colour = G.C.UI.BACKGROUND_INACTIVE
        e.config.button = nil
    end
end

G.FUNCS.on_opponent_code_change = function(e)
    if G.CONTROLLER.text_input_hook or G.opp_ext_code == '' or #G.opp_ext_code ~= 8 or G.opp_ext_code == BALATRO_VS_CTX.network.current_friendly_room_code then
        e.config.colour = G.C.UI.BACKGROUND_INACTIVE
        e.config.button = nil
    else
        e.config.colour = G.C.PURPLE
        e.config.button = 'versus_friendlies_start_pairing'
    end
end

G.FUNCS.versus_friendlies_start_pairing = function(e)
    print("Starting friendlies pairing...")
    if network_start_versus_friendlies_pairing then
        network_quit_matchmaking()
        local res = network_start_versus_friendlies_pairing(G.opp_ext_code)
        if not (res) then
            bvs_debug("Failed to connect with code " .. G.opp_ext_code)
        end
    end
end

G.FUNCS.versus_is_up_to_date = function(e)
    if BALATRO_VS_CTX then
        if BALATRO_VS_CTX.network.is_updating then
            return
        end

        if updater_check_for_update and not updater_check_for_update() then
            e.config.colour = G.C.UI.BACKGROUND_INACTIVE
            e.config.button = nil
        end

        if updater_should_update and updater_should_update() and not BALATRO_VS_CTX.network.is_updating then
            BALATRO_VS_CTX.network.is_updating = true

            on_update()
        end
    end
end

G.FUNCS.updater_update_bvs = function()
    if updater_update then
        G.FUNCS.quit()
    end
end

G.FUNCS.network_wait_for_next_action = function()
    if network_wait_for_next_action then
        network_wait_for_next_action()
    end
end

G.FUNCS.versus_matchmaking_start = function()
    if network_start_matchmaking ~= nil then
        local res = network_start_matchmaking()

        if not (res) then
            print("Failed to start matchmaking...")
        end
    end
end


G.FUNCS.network_confirm_versus_matchmaking = function()
    BALATRO_VS_CTX.network.has_confirmed_matchmaking = true
    if network_confirm_versus_matchmaking ~= nil then
        local res = network_confirm_versus_matchmaking()

        if not (res) then
            print("Failed to confirm match!")
        end
    end
end

G.FUNCS.versus_friendlies_start = function()
    if network_start_versus_friendlies ~= nil then
        local room = network_start_versus_friendlies()
        if room ~= nil and room ~= '' then
            BALATRO_VS_CTX.network.current_friendly_room_code = room
            local room_obfuscated = room:gsub('.', '*')
            G.opp_ext_code = ''
            return
            {
                n = G.UIT.ROOT,
                config = { align = "cm", minw = 10.5, minh = 6.0, padding = 0.1 },
                nodes = {
                    {
                        n = G.UIT.R,
                        config = { align = "cm", minw = 1.4, padding = 0.2 },
                        nodes = {
                            { n = G.UIT.T, config = { text = localize('b_versus_friendlies_tip1'), scale = 0.5, colour = G.C.UI.TEXT_LIGHT, shadow = true } },
                        }
                    },
                    {
                        n = G.UIT.R,
                        config = { align = "cm", minw = 1.4, padding = 0.5 },
                        nodes = {
                            { n = G.UIT.T, config = { text = room_obfuscated, scale = 1.0, colour = G.C.UI.TEXT_LIGHT, shadow = true } },
                            UIBox_button({
                                text_scale = 1.0,
                                label = { localize('b_versus_friendlies_copy_code') },
                                minw = 2,
                                minh = 1.0,
                                button = 'copy_code',
                                colour = G.C.BLUE,
                                scale = 5,
                                col = true,
                            })
                        }
                    },
                    {
                        n = G.UIT.R,
                        config = { align = "cm", minw = 1.4, padding = 0.2 },
                        nodes = {
                            { n = G.UIT.T, config = { text = localize('b_versus_friendlies_tip2'), scale = 0.5, colour = G.C.UI.TEXT_LIGHT, shadow = true } },
                        }
                    },
                    {
                        n = G.UIT.R,
                        config = { align = "cm", minw = 1.4, padding = 0 },
                        nodes = {
                            { n = G.UIT.T, config = { text = localize('b_versus_friendlies_tip3'), scale = 0.5, colour = G.C.UI.TEXT_LIGHT, shadow = true } },
                        }
                    },
                    {
                        n = G.UIT.R,
                        config = { align = "cm", minw = 0.4, padding = 1.2 },
                        nodes = {
                            create_text_input({
                                text_scale = 1.0,
                                max_length = 8,
                                minh = 1.0,
                                all_caps = false,
                                ref_table = G,
                                ref_value = 'opp_ext_code',
                                prompt_text =
                                    localize('b_versus_friendlies_enter_code')
                            }),
                            UIBox_button({
                                text_scale = 1.0,
                                label = { localize('b_versus_friendlies_paste_code') },
                                minw = 2,
                                minh = 1.0,
                                button = 'paste_seed',
                                colour = G.C.BLUE,
                                scale = 5,
                                col = true,
                            })
                        }
                    },
                    {
                        n = G.UIT.R,
                        config = { align = "cm", minw = 1.4, padding = 0.2 },
                        nodes = {
                            UIBox_button({
                                label = { localize('b_versus_friendlies_start') },
                                button = 'versus_friendlies_start_pairing',
                                minw = 2,
                                minh = 1.0,
                                colour = G.C.PURPLE,
                                scale = 1.5,
                                col = true,
                                func = 'on_opponent_code_change'
                            })
                        }
                    },
                }
            }
        end
    end
end

G.FUNCS.copy_code = function()
    if BALATRO_VS_CTX and BALATRO_VS_CTX.network and BALATRO_VS_CTX.network.current_friendly_room_code then
        love.system.setClipboardText(BALATRO_VS_CTX.network.current_friendly_room_code)
    end
end


G.FUNCS.vs_main_menu = function()
    G.FUNCS.overlay_menu {
        definition =
            create_UIBox_generic_options({
                back_func = 'exit_overlay_menu',
                contents = {
                    create_tabs(
                        {
                            scale = 1.5,
                            tabs =
                            {
                                {
                                    chosen = true,
                                    label = localize('b_versus_matchmaking'),
                                    tab_definition_function = function()
                                        return
                                        {
                                            n = G.UIT.ROOT,
                                            config = { align = "cm", padding = 0.2, colour = G.C.BLACK, r = 0.1, emboss = 0.05, minh = 6, minw = 6 },
                                            nodes = {
                                                UIBox_button { id = 'vs-srv', button = "versus_matchmaking_start", colour = G.C.PURPLE, minw = 2.65, minh = 1.35, label = { localize('b_versus_matchmaking') }, scale = 2.2, col = true },
                                            }
                                        }
                                    end
                                },
                                {
                                    label = localize('b_versus_friendlies'),
                                    tab_definition_function = function()
                                        return G.FUNCS.versus_friendlies_start()
                                    end
                                }
                            }
                        }),
                }

            })

    }
end


G.FUNCS.quit_and_return_to_vs_main_menu = function()
    if network_quit_matchmaking ~= nil then
        network_quit_matchmaking()
    end
    BALATRO_VS_CTX.network.is_error = false
    G.FUNCS.vs_main_menu()
end

G.FUNCS.lock_highlighted_card = function()
    if not BALATRO_VS_CTX or not BALATRO_VS_CTX.network.is_live then --Not in online mode
        G.FUNCS.play_cards_from_highlighted()
        return
    end

    G.buttons:remove()

    --no more alignement allowed after lock
    G.hand:lock()

    if lume.any(G.opponent_jokers.cards, function(c) return c.config.center.key == 'j_online_discard_maestro' end)
        and not has_opponent_highlighted_cards()
    then
        on_pause_timer()

        return
    end

    -- get highlighted hand
    local highlighted_cards = G.hand:get_highlighted_cards_index()

    if network_send_highlighted_card ~= nil then
        network_send_highlighted_card(highlighted_cards)
    end
end

G.FUNCS.opponent_draw_from_his_deck_to_his_hand = function(e)
    bvs_debug("Opponent drawing from his deck to his hand")

    local hand_space = e or
        math.min(#G.opponent_deck.cards, G.opponent_hand.config.card_limit - #G.opponent_hand.cards)
    delay(0.3)
    for i = 1, hand_space do
        draw_card(G.opponent_deck, G.opponent_hand, i * 100 / hand_space, 'up', true)
    end

    G.E_MANAGER:add_event(Event({
        trigger = 'after',
        no_delete = true,
        func = function()
            if G.STATE ~= G.STATES.SHOP and G.STATE ~= G.STATES.NEW_ROUND then --Timer event on regular play (so no shop and no new round)
                if BALATRO_VS_CTX.timer and not BALATRO_VS_CTX.timer:is_active() then
                    on_start_timer()                                           --Start timer
                else
                    if hand_space <= 5 and not lume.any(G.opponent_jokers.cards, function(c)
                            return c.config.center.key ==
                                'j_online_discard_maestro'
                        end) then
                        on_resume_timer() --Resume timer
                    end
                end
            end
            return true
        end
    }))
end

G.FUNCS.opponent_draw_from_his_hand_to_his_deck = function(e)
    local hand_count = #G.opponent_hand.cards
    for i = 1, hand_count do
        draw_card(G.opponent_hand, G.opponent_deck, i * 100 / hand_count, 'down', nil, nil, 0.08)
    end
end

G.FUNCS.opponent_evaluate_play = function()
    local text, disp_text, poker_hands, scoring_hand, non_loc_disp_text = G.FUNCS.get_poker_hand_info(
        G.opponent_play.cards, true)

    G.GAME.opponent_hands[text].played = G.GAME.opponent_hands[text].played + 1
    G.GAME.opponent_hands[text].played_this_round = G.GAME.opponent_hands[text].played_this_round + 1
    G.GAME.last_hand_played = text
    G.GAME.opponent_hands[text].visible = true

    --Add all the pure bonus cards to the scoring hand
    local pures = {}
    for i = 1, #G.opponent_play.cards do
        if next(opponent_find_joker('Splash')) then
            scoring_hand[i] = G.opponent_play.cards[i]
        else
            if G.opponent_play.cards[i].ability.effect == 'Stone Card' then
                local inside = false
                for j = 1, #scoring_hand do
                    if scoring_hand[j] == G.opponent_play.cards[i] then
                        inside = true
                    end
                end
                if not inside then table.insert(pures, G.opponent_play.cards[i]) end
            end
        end
    end
    for i = 1, #pures do
        table.insert(scoring_hand, pures[i])
    end
    table.sort(scoring_hand, function(a, b) return a.T.x < b.T.x end)
    delay(0.2)
    for i = 1, #scoring_hand do
        --Highlight all the cards used in scoring and play a sound indicating highlight
        highlight_card(scoring_hand[i], (i - 0.999) / 5, 'up')
    end

    local percent = 0.3
    local percent_delta = 0.08

    if G.GAME.opponent_current_round.current_hand.handname ~= disp_text then delay(0.3) end
    opponent_update_hand_text(
        {
            sound = G.GAME.opponent_current_round.current_hand.handname ~= disp_text and 'button' or nil,
            volume = 0.4,
            immediate = true,
            nopulse = nil,
            delay = G.GAME.opponent_current_round.current_hand.handname ~= disp_text and 0.4 or 0
        },
        {
            handname = disp_text,
            level = G.GAME.opponent_hands[text].level,
            mult = G.GAME.opponent_hands[text].mult,
            chips = G.GAME.opponent_hands
                [text].chips
        })

    if not G.GAME.blind:debuff_hand(G.opponent_play.cards, poker_hands, text) then
        mult = mod_mult(G.GAME.opponent_hands[text].mult)
        hand_chips = mod_chips(G.GAME.opponent_hands[text].chips)

        check_for_unlock({
            type = 'hand',
            handname = text,
            disp_text = non_loc_disp_text,
            scoring_hand = scoring_hand,
            full_hand =
                G.opponent_play.cards
        })

        delay(0.4)

        if G.GAME.first_used_hand_level and G.GAME.first_used_hand_level > 0 then
            opponent_level_up_hand(G.opponent_deck.cards[1], text, nil, G.GAME.first_used_hand_level)
            G.GAME.first_used_hand_level = nil
        end

        local hand_text_set = false
        for i = 1, #G.opponent_jokers.cards do
            --calculate the joker effects
            local effects = opponent_eval_card(G.opponent_jokers.cards[i],
                {
                    cardarea = G.opponent_jokers,
                    full_hand = G.opponent_play.cards,
                    scoring_hand = scoring_hand,
                    scoring_name = text,
                    poker_hands =
                        poker_hands,
                    before = true
                })
            if effects.jokers then
                opponent_card_eval_status_text(G.opponent_jokers.cards[i], 'jokers', nil, percent, nil, effects.jokers)
                percent = percent + percent_delta
                if effects.jokers.level_up then
                    opponent_level_up_hand(G.opponent_jokers.cards[i], text)
                end
            end
        end

        mult = mod_mult(G.GAME.opponent_hands[text].mult)
        hand_chips = mod_chips(G.GAME.opponent_hands[text].chips)

        local modded = false

        mult, hand_chips, modded = G.GAME.blind:modify_hand(G.opponent_play.cards, poker_hands, text, mult, hand_chips)
        mult, hand_chips = mod_mult(mult), mod_chips(hand_chips)
        if modded then
            opponent_update_hand_text({ sound = 'chips2', modded = modded },
                { chips = hand_chips, mult = mult })
        end
        for i = 1, #scoring_hand do
            --add cards played to list
            if scoring_hand[i].ability.effect ~= 'Stone Card' then
                G.GAME.cards_played[scoring_hand[i].base.value].total = G.GAME.cards_played[scoring_hand[i].base.value]
                    .total + 1
                G.GAME.cards_played[scoring_hand[i].base.value].suits[scoring_hand[i].base.suit] = true
            end
            --if card is debuffed
            if scoring_hand[i].debuff then
                G.GAME.blind.triggered = true
                G.E_MANAGER:add_event(Event({
                    trigger = 'immediate',
                    func = (function()
                        G.HUD_blind:get_UIE_by_ID('HUD_blind_debuff_1'):juice_up(0.3, 0)
                        G.HUD_blind:get_UIE_by_ID('HUD_blind_debuff_2'):juice_up(0.3, 0)
                        G.GAME.blind:juice_up(); return true
                    end)
                }))
                opponent_card_eval_status_text(scoring_hand[i], 'debuff')
            else
                --Check for play doubling
                local reps = { 1 }

                --From Red seal
                local eval = opponent_eval_card(scoring_hand[i],
                    {
                        repetition_only = true,
                        cardarea = G.opponent_play,
                        full_hand = G.opponent_play.cards,
                        scoring_hand = scoring_hand,
                        scoring_name =
                            text,
                        poker_hands = poker_hands,
                        repetition = true
                    })
                if next(eval) then
                    for h = 1, eval.seals.repetitions do
                        reps[#reps + 1] = eval
                    end
                end
                --From jokers
                for j = 1, #G.opponent_jokers.cards do
                    --calculate the joker effects
                    local eval = opponent_eval_card(G.opponent_jokers.cards[j],
                        {
                            cardarea = G.opponent_play,
                            full_hand = G.opponent_play.cards,
                            scoring_hand = scoring_hand,
                            scoring_name = text,
                            poker_hands =
                                poker_hands,
                            other_card = scoring_hand[i],
                            repetition = true
                        })
                    if next(eval) and eval.jokers then
                        for h = 1, eval.jokers.repetitions do
                            reps[#reps + 1] = eval
                        end
                    end
                end
                for j = 1, #reps do
                    percent = percent + percent_delta
                    if reps[j] ~= 1 then
                        opponent_card_eval_status_text((reps[j].jokers or reps[j].seals).card, 'jokers', nil, nil, nil,
                            (reps[j].jokers or reps[j].seals))
                    end

                    --calculate the hand effects
                    local effects = { opponent_eval_card(scoring_hand[i],
                        {
                            cardarea = G.opponent_play,
                            full_hand = G.opponent_play.cards,
                            scoring_hand = scoring_hand,
                            poker_hand =
                                text
                        }) }
                    for k = 1, #G.opponent_jokers.cards do
                        --calculate the joker individual card effects
                        local eval = G.opponent_jokers.cards[k]:opponent_calculate_joker({
                            cardarea = G.opponent_play,
                            full_hand = G.opponent_play.cards,
                            scoring_hand =
                                scoring_hand,
                            scoring_name = text,
                            poker_hands = poker_hands,
                            other_card = scoring_hand[i],
                            individual = true
                        })
                        if eval then
                            table.insert(effects, eval)
                        end
                    end
                    scoring_hand[i].lucky_trigger = nil

                    for ii = 1, #effects do
                        --If chips added, do chip add event and add the chips to the total
                        if effects[ii].chips then
                            if effects[ii].card then juice_card(effects[ii].card) end
                            hand_chips = mod_chips(hand_chips + effects[ii].chips)
                            opponent_update_hand_text({ delay = 0 }, { chips = hand_chips })
                            opponent_card_eval_status_text(scoring_hand[i], 'chips', effects[ii].chips, percent)
                        end

                        --If mult added, do mult add event and add the mult to the total
                        if effects[ii].mult then
                            if effects[ii].card then juice_card(effects[ii].card) end
                            mult = mod_mult(mult + effects[ii].mult)
                            opponent_update_hand_text({ delay = 0 }, { mult = mult })
                            opponent_card_eval_status_text(scoring_hand[i], 'mult', effects[ii].mult, percent)
                        end

                        --If play dollars added, add dollars to total
                        if effects[ii].p_dollars then
                            if effects[ii].card then juice_card(effects[ii].card) end
                            opponent_ease_dollars(effects[ii].p_dollars)
                            opponent_card_eval_status_text(scoring_hand[i], 'dollars', effects[ii].p_dollars, percent)
                        end

                        --If dollars added, add dollars to total
                        if effects[ii].dollars then
                            if effects[ii].card then juice_card(effects[ii].card) end
                            opponent_ease_dollars(effects[ii].dollars)
                            opponent_card_eval_status_text(scoring_hand[i], 'dollars', effects[ii].dollars, percent)
                        end

                        --Any extra effects
                        if effects[ii].extra then
                            if effects[ii].card then juice_card(effects[ii].card) end
                            local extras = { mult = false, hand_chips = false }
                            if effects[ii].extra.mult_mod then
                                mult = mod_mult(mult + effects[ii].extra.mult_mod); extras.mult = true
                            end
                            if effects[ii].extra.chip_mod then
                                hand_chips = mod_chips(hand_chips + effects[ii].extra.chip_mod); extras.hand_chips = true
                            end
                            if effects[ii].extra.swap then
                                local old_mult = mult
                                mult = mod_mult(hand_chips)
                                hand_chips = mod_chips(old_mult)
                                extras.hand_chips = true; extras.mult = true
                            end
                            if effects[ii].extra.func then effects[ii].extra.func() end
                            opponent_update_hand_text({ delay = 0 },
                                { chips = extras.hand_chips and hand_chips, mult = extras.mult and mult })
                            opponent_card_eval_status_text(scoring_hand[i], 'extra', nil, percent, nil, effects[ii]
                                .extra)
                        end

                        --If x_mult added, do mult add event and mult the mult to the total
                        if effects[ii].x_mult then
                            if effects[ii].card then juice_card(effects[ii].card) end
                            mult = mod_mult(mult * effects[ii].x_mult)
                            opponent_update_hand_text({ delay = 0 }, { mult = mult })
                            opponent_card_eval_status_text(scoring_hand[i], 'x_mult', effects[ii].x_mult, percent)
                        end

                        --calculate the card edition effects
                        if effects[ii].edition then
                            hand_chips = mod_chips(hand_chips + (effects[ii].edition.chip_mod or 0))
                            mult = mult + (effects[ii].edition.mult_mod or 0)
                            mult = mod_mult(mult * (effects[ii].edition.x_mult_mod or 1))
                            opponent_update_hand_text({ delay = 0 }, {
                                chips = effects[ii].edition.chip_mod and hand_chips or nil,
                                mult = (effects[ii].edition.mult_mod or effects[ii].edition.x_mult_mod) and mult or nil,
                            })
                            opponent_card_eval_status_text(scoring_hand[i], 'extra', nil, percent, nil, {
                                message = (effects[ii].edition.chip_mod and localize { type = 'variable', key = 'a_chips', vars = { effects[ii].edition.chip_mod } }) or
                                    (effects[ii].edition.mult_mod and localize { type = 'variable', key = 'a_mult', vars = { effects[ii].edition.mult_mod } }) or
                                    (effects[ii].edition.x_mult_mod and localize { type = 'variable', key = 'a_xmult', vars = { effects[ii].edition.x_mult_mod } }),
                                chip_mod = effects[ii].edition.chip_mod,
                                mult_mod = effects[ii].edition.mult_mod,
                                x_mult_mod = effects[ii].edition.x_mult_mod,
                                colour = G.C.DARK_EDITION,
                                edition = true
                            })
                        end
                    end
                end
            end
        end

        delay(0.3)
        local mod_percent = false
        for i = 1, #G.opponent_hand.cards do
            if mod_percent then percent = percent + percent_delta end
            mod_percent = false

            --Check for hand doubling
            local reps = { 1 }
            local j = 1
            while j <= #reps do
                if reps[j] ~= 1 then
                    opponent_card_eval_status_text((reps[j].jokers or reps[j].seals).card, 'jokers', nil, nil, nil,
                        (reps[j].jokers or reps[j].seals))
                    percent = percent + percent_delta
                end

                --calculate the hand effects
                local effects = { opponent_eval_card(G.opponent_hand.cards[i],
                    {
                        cardarea = G.opponent_hand,
                        full_hand = G.opponent_play.cards,
                        scoring_hand = scoring_hand,
                        scoring_name = text,
                        poker_hands =
                            poker_hands
                    }) }

                for k = 1, #G.opponent_jokers.cards do
                    --calculate the joker individual card effects
                    local eval = G.opponent_jokers.cards[k]:opponent_calculate_joker({
                        cardarea = G.opponent_hand,
                        full_hand = G.opponent_play.cards,
                        scoring_hand =
                            scoring_hand,
                        scoring_name = text,
                        poker_hands = poker_hands,
                        other_card = G.opponent_hand.cards[i],
                        individual = true
                    })
                    if eval then
                        mod_percent = true
                        table.insert(effects, eval)
                    end
                end

                if reps[j] == 1 then
                    --Check for hand doubling

                    --From Red seal
                    local eval = opponent_eval_card(G.opponent_hand.cards[i],
                        {
                            repetition_only = true,
                            cardarea = G.opponent_hand,
                            full_hand = G.opponent_play.cards,
                            scoring_hand =
                                scoring_hand,
                            scoring_name = text,
                            poker_hands = poker_hands,
                            repetition = true,
                            card_effects =
                                effects
                        })
                    if next(eval) and (next(effects[1]) or #effects > 1) then
                        for h = 1, eval.seals.repetitions do
                            reps[#reps + 1] = eval
                        end
                    end

                    --From Joker
                    for j = 1, #G.opponent_jokers.cards do
                        --calculate the joker effects
                        local eval = opponent_eval_card(G.opponent_jokers.cards[j],
                            {
                                cardarea = G.opponent_hand,
                                full_hand = G.opponent_play.cards,
                                scoring_hand = scoring_hand,
                                scoring_name =
                                    text,
                                poker_hands = poker_hands,
                                other_card = G.opponent_hand.cards[i],
                                repetition = true,
                                card_effects =
                                    effects
                            })
                        if next(eval) then
                            for h = 1, eval.jokers.repetitions do
                                reps[#reps + 1] = eval
                            end
                        end
                    end
                end

                for ii = 1, #effects do
                    --if this effect came from a joker
                    if effects[ii].card then
                        mod_percent = true
                        G.E_MANAGER:add_event(Event({
                            trigger = 'immediate',
                            func = (function()
                                effects[ii].card:juice_up(0.7); return true
                            end)
                        }))
                    end

                    --If hold mult added, do hold mult add event and add the mult to the total

                    --If dollars added, add dollars to total
                    if effects[ii].dollars then
                        opponent_ease_dollars(effects[ii].dollars)
                        opponent_card_eval_status_text(G.opponent_hand.cards[i], 'dollars', effects[ii].dollars, percent)
                    end

                    if effects[ii].h_mult then
                        mod_percent = true
                        mult = mod_mult(mult + effects[ii].h_mult)
                        opponent_update_hand_text({ delay = 0 }, { mult = mult })
                        opponent_card_eval_status_text(G.opponent_hand.cards[i], 'h_mult', effects[ii].h_mult, percent)
                    end

                    if effects[ii].x_mult then
                        mod_percent = true
                        mult = mod_mult(mult * effects[ii].x_mult)
                        opponent_update_hand_text({ delay = 0 }, { mult = mult })
                        opponent_card_eval_status_text(G.opponent_hand.cards[i], 'x_mult', effects[ii].x_mult, percent)
                    end

                    if effects[ii].message then
                        mod_percent = true
                        opponent_update_hand_text({ delay = 0 }, { mult = mult })
                        opponent_card_eval_status_text(G.opponent_hand.cards[i], 'extra', nil, percent, nil, effects[ii])
                    end
                end
                j = j + 1
            end
        end
        --+++++++++++++++++++++++++++++++++++++++++++++++++++++++++--
        --Joker Effects
        --+++++++++++++++++++++++++++++++++++++++++++++++++++++++++--
        percent = percent + percent_delta
        for i = 1, #G.opponent_jokers.cards + #G.opponent_consumeables.cards do
            local _card = G.opponent_jokers.cards[i] or G.opponent_consumeables.cards[i - #G.opponent_jokers.cards]
            --calculate the joker edition effects
            local edition_effects = opponent_eval_card(_card,
                {
                    cardarea = G.opponent_jokers,
                    full_hand = G.opponent_play.cards,
                    scoring_hand = scoring_hand,
                    scoring_name = text,
                    poker_hands =
                        poker_hands,
                    edition = true
                })
            if edition_effects.jokers then
                edition_effects.jokers.edition = true
                if edition_effects.jokers.chip_mod then
                    hand_chips = mod_chips(hand_chips + edition_effects.jokers.chip_mod)
                    opponent_update_hand_text({ delay = 0 }, { chips = hand_chips })
                    opponent_card_eval_status_text(_card, 'jokers', nil, percent, nil, {
                        message = localize { type = 'variable', key = 'a_chips', vars = { edition_effects.jokers.chip_mod } },
                        chip_mod = edition_effects.jokers.chip_mod,
                        colour = G.C.EDITION,
                        edition = true
                    })
                end
                if edition_effects.jokers.mult_mod then
                    mult = mod_mult(mult + edition_effects.jokers.mult_mod)
                    opponent_update_hand_text({ delay = 0 }, { mult = mult })
                    opponent_card_eval_status_text(_card, 'jokers', nil, percent, nil, {
                        message = localize { type = 'variable', key = 'a_mult', vars = { edition_effects.jokers.mult_mod } },
                        mult_mod = edition_effects.jokers.mult_mod,
                        colour = G.C.DARK_EDITION,
                        edition = true
                    })
                end
                percent = percent + percent_delta
            end

            --calculate the joker effects
            local effects = opponent_eval_card(_card,
                {
                    cardarea = G.opponent_jokers,
                    full_hand = G.opponent_play.cards,
                    scoring_hand = scoring_hand,
                    scoring_name = text,
                    poker_hands =
                        poker_hands,
                    joker_main = true
                })

            --Any Joker effects
            if effects.jokers then
                local extras = { mult = false, hand_chips = false }
                if effects.jokers.mult_mod then
                    mult = mod_mult(mult + effects.jokers.mult_mod); extras.mult = true
                end
                if effects.jokers.chip_mod then
                    hand_chips = mod_chips(hand_chips + effects.jokers.chip_mod); extras.hand_chips = true
                end
                if effects.jokers.Xmult_mod then
                    mult = mod_mult(mult * effects.jokers.Xmult_mod); extras.mult = true
                end
                opponent_update_hand_text({ delay = 0 },
                    { chips = extras.hand_chips and hand_chips, mult = extras.mult and mult })
                opponent_card_eval_status_text(_card, 'jokers', nil, percent, nil, effects.jokers)
                percent = percent + percent_delta
            end

            --Joker on Joker effects
            for _, v in ipairs(G.opponent_jokers.cards) do
                local effect = v:opponent_calculate_joker { full_hand = G.opponent_play.cards, scoring_hand = scoring_hand, scoring_name = text, poker_hands = poker_hands, other_joker = _card }
                if effect then
                    local extras = { mult = false, hand_chips = false }
                    if effect.mult_mod then
                        mult = mod_mult(mult + effect.mult_mod); extras.mult = true
                    end
                    if effect.chip_mod then
                        hand_chips = mod_chips(hand_chips + effect.chip_mod); extras.hand_chips = true
                    end
                    if effect.Xmult_mod then
                        mult = mod_mult(mult * effect.Xmult_mod); extras.mult = true
                    end
                    if extras.mult or extras.hand_chips then
                        opponent_update_hand_text({ delay = 0 },
                            { chips = extras.hand_chips and hand_chips, mult = extras.mult and mult })
                    end
                    if extras.mult or extras.hand_chips then
                        opponent_card_eval_status_text(v, 'jokers', nil, percent, nil,
                            effect)
                    end
                    percent = percent + percent_delta
                end
            end

            if edition_effects.jokers then
                if edition_effects.jokers.x_mult_mod then
                    mult = mod_mult(mult * edition_effects.jokers.x_mult_mod)
                    opponent_update_hand_text({ delay = 0 }, { mult = mult })
                    opponent_card_eval_status_text(_card, 'jokers', nil, percent, nil, {
                        message = localize { type = 'variable', key = 'a_xmult', vars = { edition_effects.jokers.x_mult_mod } },
                        x_mult_mod = edition_effects.jokers.x_mult_mod,
                        colour = G.C.EDITION,
                        edition = true
                    })
                end
                percent = percent + percent_delta
            end
        end

        local nu_chip, nu_mult = G.GAME.selected_back:trigger_effect { context = 'final_scoring_step', chips = hand_chips, mult = mult }
        mult = mod_mult(nu_mult or mult)
        hand_chips = mod_chips(nu_chip or hand_chips)

        local cards_destroyed = {}
        for i = 1, #scoring_hand do
            local destroyed = nil
            --un-highlight all cards
            highlight_card(scoring_hand[i], (i - 0.999) / (#scoring_hand - 0.998), 'down')

            for j = 1, #G.opponent_jokers.cards do
                destroyed = G.opponent_jokers.cards[j]:opponent_calculate_joker({
                    destroying_card = scoring_hand[i],
                    full_hand = G.opponent_play
                        .cards
                })
                if destroyed then break end
            end

            if scoring_hand[i].ability.name == 'Glass Card' and not scoring_hand[i].debuff and pseudorandom('glass', nil, nil, true) < G.GAME.opponent_probabilities.normal / scoring_hand[i].ability.extra then
                destroyed = true
            end

            if destroyed then
                if scoring_hand[i].ability.name == 'Glass Card' then
                    scoring_hand[i].shattered = true
                else
                    scoring_hand[i].destroyed = true
                end
                cards_destroyed[#cards_destroyed + 1] = scoring_hand[i]
            end
        end
        for j = 1, #G.opponent_jokers.cards do
            opponent_eval_card(G.opponent_jokers.cards[j],
                { cardarea = G.opponent_jokers, remove_playing_cards = true, removed = cards_destroyed })
        end

        local glass_shattered = {}
        for k, v in ipairs(cards_destroyed) do
            if v.shattered then glass_shattered[#glass_shattered + 1] = v end
        end

        check_for_unlock { type = 'shatter', shattered = glass_shattered }

        for i = 1, #cards_destroyed do
            G.E_MANAGER:add_event(Event({
                func = function()
                    if cards_destroyed[i].ability.name == 'Glass Card' then
                        cards_destroyed[i]:shatter()
                    else
                        cards_destroyed[i]:start_dissolve()
                    end
                    return true
                end
            }))
        end
    else
        mult = mod_mult(0)
        hand_chips = mod_chips(0)
        G.E_MANAGER:add_event(Event({
            trigger = 'immediate',
            func = (function()
                G.HUD_blind:get_UIE_by_ID('HUD_blind_debuff_1'):juice_up(0.3, 0)
                G.HUD_blind:get_UIE_by_ID('HUD_blind_debuff_2'):juice_up(0.3, 0)
                G.GAME.blind:juice_up()
                G.E_MANAGER:add_event(Event({
                    trigger = 'after',
                    delay = 0.06 * G.SETTINGS.GAMESPEED,
                    blockable = false,
                    blocking = false,
                    func = function()
                        play_sound('tarot2', 0.76, 0.4); return true
                    end
                }))
                play_sound('tarot2', 1, 0.4)
                return true
            end)
        }))

        play_area_status_text("Not Allowed!") --localize('k_not_allowed_ex'), true)
        --+++++++++++++++++++++++++++++++++++++++++++++++++++++++++--
        --Joker Debuff Effects
        --+++++++++++++++++++++++++++++++++++++++++++++++++++++++++--
        for i = 1, #G.opponent_jokers.cards do
            --calculate the joker effects
            local effects = opponent_eval_card(G.opponent_jokers.cards[i],
                {
                    cardarea = G.opponent_jokers,
                    full_hand = G.opponent_play.cards,
                    scoring_hand = scoring_hand,
                    scoring_name = text,
                    poker_hands =
                        poker_hands,
                    debuffed_hand = true
                })

            --Any Joker effects
            if effects.jokers then
                opponent_card_eval_status_text(G.opponent_jokers.cards[i], 'jokers', nil, percent, nil, effects.jokers)
                percent = percent + percent_delta
            end
        end
    end

    G.E_MANAGER:add_event(Event({
        trigger = 'after',
        delay = 0.4,
        func = (function()
            opponent_update_hand_text({ delay = 0, immediate = true },
                { mult = 0, chips = 0, chip_total = math.floor(hand_chips * mult), level = '', handname = '' }); play_sound(
                'button', 0.9, 0.6); return true
        end)
    }))
    opponent_check_and_set_high_score('hand', hand_chips * mult)

    if hand_chips * mult > 0 then
        delay(0.8)
        G.E_MANAGER:add_event(Event({
            trigger = 'immediate',
            func = (function()
                play_sound('chips2'); return true
            end)
        }))
    end
    G.E_MANAGER:add_event(Event({
        trigger = 'ease',
        blocking = false,
        ref_table = G.GAME,
        ref_value = 'opponent_chips',
        ease_to = G.GAME.opponent_chips + math.floor(hand_chips * mult),
        delay = 0.5,
        func = (function(t) return math.floor(t) end)
    }))
    G.E_MANAGER:add_event(Event({
        trigger = 'ease',
        blocking = true,
        ref_table = G.GAME.opponent_current_round.current_hand,
        ref_value = 'chip_total',
        ease_to = 0,
        delay = 0.5,
        func = (function(t) return math.floor(t) end)
    }))
    G.E_MANAGER:add_event(Event({
        trigger = 'immediate',
        func = (function()
            G.GAME.opponent_current_round.current_hand.handname = ''; return true
        end)
    }))
    delay(0.3)

    for i = 1, #G.opponent_jokers.cards do
        --calculate the joker after hand played effects
        local effects = opponent_eval_card(G.opponent_jokers.cards[i],
            {
                cardarea = G.opponent_jokers,
                full_hand = G.opponent_play.cards,
                scoring_hand = scoring_hand,
                scoring_name = text,
                poker_hands =
                    poker_hands,
                after = true
            })
        if effects.jokers then
            opponent_card_eval_status_text(G.opponent_jokers.cards[i], 'jokers', nil, percent, nil, effects.jokers)
            percent = percent + percent_delta
        end
    end

    G.E_MANAGER:add_event(Event({
        trigger = 'immediate',
        func = (function()
            if G.GAME.modifiers.debuff_played_cards then
                for k, v in ipairs(scoring_hand) do v.ability.perma_debuff = true end
            end
            return true
        end)
    }))
end


function opponent_check_and_set_high_score(score, amt)
    if not amt or type(amt) ~= 'number' then return end
    if G.GAME.opponent_round_scores[score] and math.floor(amt) > G.GAME.opponent_round_scores[score].amt then
        G.GAME.opponent_round_scores[score].amt = math.floor(amt)
    end
end

function opponent_level_up_hand(card, hand, instant, amount)
    amount = amount or 1
    G.GAME.opponent_hands[hand].level = math.max(0, G.GAME.opponent_hands[hand].level + amount)
    G.GAME.opponent_hands[hand].mult = math.max(
        G.GAME.opponent_hands[hand].s_mult + G.GAME.opponent_hands[hand].l_mult * (G.GAME.opponent_hands[hand].level - 1),
        1)
    G.GAME.opponent_hands[hand].chips = math.max(
        G.GAME.opponent_hands[hand].s_chips +
        G.GAME.opponent_hands[hand].l_chips * (G.GAME.opponent_hands[hand].level - 1), 0)
    if not instant then
        G.E_MANAGER:add_event(Event({
            trigger = 'after',
            delay = 0.2,
            func = function()
                play_sound('tarot1')
                if card then card:juice_up(0.8, 0.5) end
                G.TAROT_INTERRUPT_PULSE = true
                return true
            end
        }))
        opponent_update_hand_text({ delay = 0 }, { mult = G.GAME.opponent_hands[hand].mult, StatusText = true })
        G.E_MANAGER:add_event(Event({
            trigger = 'after',
            delay = 0.9,
            func = function()
                play_sound('tarot1')
                if card then card:juice_up(0.8, 0.5) end
                return true
            end
        }))
        opponent_update_hand_text({ delay = 0 }, { chips = G.GAME.opponent_hands[hand].chips, StatusText = true })
        G.E_MANAGER:add_event(Event({
            trigger = 'after',
            delay = 0.9,
            func = function()
                play_sound('tarot1')
                if card then card:juice_up(0.8, 0.5) end
                G.TAROT_INTERRUPT_PULSE = nil
                return true
            end
        }))
        opponent_update_hand_text({ sound = 'button', volume = 0.7, pitch = 0.9, delay = 0 },
            { level = G.GAME.opponent_hands[hand].level })
        delay(1.3)
    end
    G.E_MANAGER:add_event(Event({
        trigger = 'immediate',
        func = (function()
            check_for_unlock { type = 'upgrade_hand', hand = hand, level = G.GAME.opponent_hands[hand].level }
            return true
        end)
    }))
end

function opponent_update_hand_text(config, vals)
    G.E_MANAGER:add_event(Event({ --This is the Hand name text for the poker hand
        trigger = 'before',
        blockable = not config.immediate,
        delay = config.delay or 0.8,
        func = function()
            local col = G.C.GREEN
            if vals.chips and G.GAME.opponent_current_round.current_hand.chips ~= vals.chips then
                local delta = (type(vals.chips) == 'number' and type(G.GAME.opponent_current_round.current_hand.chips) == 'number') and
                    (vals.chips - G.GAME.opponent_current_round.current_hand.chips) or 0
                if delta < 0 then
                    delta = '' .. delta; col = G.C.RED
                elseif delta > 0 then
                    delta = '+' .. delta
                else
                    delta = '' .. delta
                end
                if type(vals.chips) == 'string' then delta = vals.chips end
                G.GAME.opponent_current_round.current_hand.chips = vals.chips
                G.opponent_hand_text_area.chips:update(0)
                if vals.StatusText then
                    attention_text({
                        text = delta,
                        scale = 0.8,
                        hold = 1,
                        cover = G.opponent_hand_text_area.chips.parent,
                        cover_colour = mix_colours(G.C.CHIPS, col, 0.1),
                        emboss = 0.05,
                        align = 'cm',
                        cover_align = 'cr'
                    })
                end
            end
            if vals.mult and G.GAME.opponent_current_round.current_hand.mult ~= vals.mult then
                local delta = (type(vals.mult) == 'number' and type(G.GAME.opponent_current_round.current_hand.mult) == 'number') and
                    (vals.mult - G.GAME.opponent_current_round.current_hand.mult) or 0
                if delta < 0 then
                    delta = '' .. delta; col = G.C.RED
                elseif delta > 0 then
                    delta = '+' .. delta
                else
                    delta = '' .. delta
                end
                if type(vals.mult) == 'string' then delta = vals.mult end
                G.GAME.opponent_current_round.current_hand.mult = vals.mult
                G.opponent_hand_text_area.mult:update(0)
                if vals.StatusText then
                    attention_text({
                        text = delta,
                        scale = 0.8,
                        hold = 1,
                        cover = G.opponent_hand_text_area.mult.parent,
                        cover_colour = mix_colours(G.C.MULT, col, 0.1),
                        emboss = 0.05,
                        align = 'cm',
                        cover_align = 'cl'
                    })
                end
                if not G.TAROT_INTERRUPT then G.opponent_hand_text_area.mult:juice_up() end
            end
            if vals.handname and G.GAME.opponent_current_round.current_hand.handname ~= vals.handname then
                G.GAME.opponent_current_round.current_hand.handname = vals.handname
                if not config.nopulse then
                    G.opponent_hand_text_area.handname.config.object:pulse(0.2)
                end
            end
            if vals.chip_total then
                G.GAME.opponent_current_round.current_hand.chip_total = vals.chip_total; G.opponent_hand_text_area
                    .chip_total
                    .config
                    .object:pulse(0.5)
            end
            if vals.level and G.GAME.opponent_current_round.current_hand.hand_level ~= ' ' .. localize('k_lvl') .. tostring(vals.level) then
                if vals.level == '' then
                    G.GAME.opponent_current_round.current_hand.hand_level = vals.level
                else
                    G.GAME.opponent_current_round.current_hand.hand_level = ' ' ..
                        localize('k_lvl') .. tostring(vals.level)
                    if type(vals.level) == 'number' then
                        G.opponent_hand_text_area.hand_level.config.colour = G.C.HAND_LEVELS[math.min(vals.level, 7)]
                    else
                        G.opponent_hand_text_area.hand_level.config.colour = G.C.HAND_LEVELS[1]
                    end
                    G.opponent_hand_text_area.hand_level:juice_up()
                end
            end
            if config.sound and not config.modded then play_sound(config.sound, config.pitch or 1, config.volume or 1) end
            if config.modded then
                G.HUD_blind:get_UIE_by_ID('HUD_blind_debuff_1'):juice_up(0.3, 0)
                G.HUD_blind:get_UIE_by_ID('HUD_blind_debuff_2'):juice_up(0.3, 0)
                G.GAME.blind:juice_up()
                G.E_MANAGER:add_event(Event({
                    trigger = 'after',
                    delay = 0.06 * G.SETTINGS.GAMESPEED,
                    blockable = false,
                    blocking = false,
                    func = function()
                        play_sound('tarot2', 0.76, 0.4); return true
                    end
                }))
                play_sound('tarot2', 1, 0.4)
            end
            return true
        end
    }))
end

function opponent_card_eval_status_text(card, eval_type, amt, percent, dir, extra)
    percent = percent or (0.9 + 0.2 * math.random())
    if dir == 'down' then
        percent = 1 - percent
    end

    if extra and extra.focus then card = extra.focus end

    local text = ''
    local sound = nil
    local volume = 1
    local card_aligned = 'bm'
    local y_off = 0.15 * G.CARD_H
    if card.area == G.opponent_jokers or card.area == G.opponent_consumeables then
        y_off = 0.05 * card.T.h
    elseif card.area == G.opponent_hand then
        y_off = -0.05 * G.CARD_H
        card_aligned = 'tm'
    elseif card.area == G.opponent_play then
        y_off = -0.05 * G.CARD_H
        card_aligned = 'tm'
    elseif card.jimbo then
        y_off = -0.05 * G.CARD_H
        card_aligned = 'tm'
    end
    local config = {}
    local delay = 0.65
    local colour = config.colour or (extra and extra.colour) or (G.C.FILTER)
    local extrafunc = nil

    if eval_type == 'debuff' then
        sound = 'cancel'
        amt = 1
        colour = G.C.RED
        config.scale = 0.6
        text = localize('k_debuffed')
    elseif eval_type == 'chips' then
        sound = 'chips1'
        amt = amt
        colour = G.C.CHIPS
        text = localize { type = 'variable', key = 'a_chips', vars = { amt } }
        delay = 0.6
    elseif eval_type == 'mult' then
        sound = 'multhit1' --'other1'
        amt = amt
        text = localize { type = 'variable', key = 'a_mult', vars = { amt } }
        colour = G.C.MULT
        config.type = 'fade'
        config.scale = 0.7
    elseif (eval_type == 'x_mult') or (eval_type == 'h_x_mult') then
        sound = 'multhit2'
        volume = 0.7
        amt = amt
        text = localize { type = 'variable', key = 'a_xmult', vars = { amt } }
        colour = G.C.XMULT
        config.type = 'fade'
        config.scale = 0.7
    elseif eval_type == 'h_mult' then
        sound = 'multhit1'
        amt = amt
        text = localize { type = 'variable', key = 'a_mult', vars = { amt } }
        colour = G.C.MULT
        config.type = 'fade'
        config.scale = 0.7
    elseif eval_type == 'dollars' then
        sound = 'coin3'
        amt = amt
        text = (amt < -0.01 and '-' or '') .. localize("$") .. tostring(math.abs(amt))
        colour = amt < -0.01 and G.C.RED or G.C.MONEY
    elseif eval_type == 'swap' then
        sound = 'generic1'
        amt = amt
        text = localize('k_swapped_ex')
        colour = G.C.PURPLE
    elseif eval_type == 'extra' or eval_type == 'jokers' then
        sound = extra.edition and 'foil2' or extra.mult_mod and 'multhit1' or extra.Xmult_mod and 'multhit2' or
            'generic1'
        if extra.edition then
            colour = G.C.DARK_EDITION
        end
        volume = extra.edition and 0.3 or sound == 'multhit2' and 0.7 or 1
        delay = extra.delay or 0.75
        amt = 1
        text = extra.message or text
        if not extra.edition and (extra.mult_mod or extra.Xmult_mod) then
            colour = G.C.MULT
        end
        if extra.chip_mod then
            config.type = 'fall'
            colour = G.C.CHIPS
            config.scale = 0.7
        elseif extra.swap then
            config.type = 'fall'
            colour = G.C.PURPLE
            config.scale = 0.7
        else
            config.type = 'fall'
            config.scale = 0.7
        end
    end
    delay = delay * 1.25

    if amt > 0 or amt < 0 then
        if extra and extra.instant then
            if extrafunc then extrafunc() end
            attention_text({
                text = text,
                scale = config.scale or 1,
                hold = delay - 0.2,
                backdrop_colour = colour,
                align = card_aligned,
                major = card,
                offset = { x = 0, y = y_off }
            })
            play_sound(sound, 0.8 + percent * 0.2, volume)
            if not extra or not extra.no_juice then
                card:juice_up(0.6, 0.1)
                G.ROOM.jiggle = G.ROOM.jiggle + 0.7
            end
        else
            G.E_MANAGER:add_event(Event({ --Add bonus chips from this card
                trigger = 'before',
                delay = delay,
                func = function()
                    if extrafunc then extrafunc() end
                    attention_text({
                        text = text,
                        scale = config.scale or 1,
                        hold = delay - 0.2,
                        backdrop_colour = colour,
                        align = card_aligned,
                        major = card,
                        offset = { x = 0, y = y_off }
                    })
                    play_sound(sound, 0.8 + percent * 0.2, volume)
                    if not extra or not extra.no_juice then
                        card:juice_up(0.6, 0.1)
                        G.ROOM.jiggle = G.ROOM.jiggle + 0.7
                    end
                    return true
                end
            }))
        end
    end
    if extra and extra.playing_cards_created then
        opponent_playing_card_joker_effects(extra.playing_cards_created)
    end
end

function opponent_playing_card_joker_effects(cards)
    for i = 1, #G.opponent_jokers.cards do
        G.opponent_jokers.cards[i]:opponent_calculate_joker({ playing_card_added = true, cards = cards })
    end
end

G.FUNCS.opponent_hand_text_UI_set = function(e)
    if G.GAME.opponent_current_round.current_hand.handname ~= G.GAME.opponent_current_round.current_hand.handname_text then
        G.GAME.opponent_current_round.current_hand.handname_text = G.GAME.opponent_current_round.current_hand.handname
        if G.GAME.opponent_current_round.current_hand.handname:len() >= 13 then
            e.config.object.scale = 12 * 0.56 / G.GAME.opponent_current_round.current_hand.handname:len()
        else
            e.config.object.scale = 2.4 / math.sqrt(G.GAME.opponent_current_round.current_hand.handname:len() + 5)
        end
        e.config.object:update_text()
    end
end

G.FUNCS.opponent_hand_chip_total_UI_set = function(e)
    if G.GAME.opponent_current_round.current_hand.chip_total < 1 then
        G.GAME.opponent_current_round.current_hand.chip_total_text = ''
    else
        local new_chip_total_text = number_format(G.GAME.opponent_current_round.current_hand.chip_total)
        if new_chip_total_text ~= G.GAME.opponent_current_round.current_hand.chip_total_text then
            e.config.object.scale = scale_number(G.GAME.opponent_current_round.current_hand.chip_total, 0.95, 100000000)

            G.GAME.opponent_current_round.current_hand.chip_total_text = new_chip_total_text
            if not G.ARGS.opponent_hand_chip_total_UI_set or G.ARGS.opponent_hand_chip_total_UI_set < G.GAME.opponent_current_round.current_hand.chip_total then
                G.FUNCS.text_super_juice(e, math.floor(math.log10(G.GAME.opponent_current_round.current_hand.chip_total)))
            end
            G.ARGS.opponent_hand_chip_total_UI_set = G.GAME.opponent_current_round.current_hand.chip_total
            --e.UIBox:recalculate()
        end
    end
end

G.FUNCS.opponent_hand_chip_UI_set = function(e)
    local new_chip_text = number_format(G.GAME.opponent_current_round.current_hand.chips)
    if new_chip_text ~= G.GAME.opponent_current_round.current_hand.chip_text then
        G.GAME.opponent_current_round.current_hand.chip_text = new_chip_text
        e.config.object.scale = scale_number(G.GAME.opponent_current_round.current_hand.chips, 0.9, 1000)
        e.config.object:update_text()
        if not G.TAROT_INTERRUPT_PULSE then
            G.FUNCS.text_super_juice(e,
                math.max(0,
                    math.floor(math.log10(type(G.GAME.opponent_current_round.current_hand.chips) == 'number' and
                        G.GAME.opponent_current_round.current_hand.chips or 1))))
        end
    end
end

G.FUNCS.opponent_flame_handler = function(e)
    G.C.UI_CHIPLICK = G.C.UI_CHIPLICK or { 1, 1, 1, 1 }
    G.C.UI_MULTLICK = G.C.UI_MULTLICK or { 1, 1, 1, 1 }
    for i = 1, 3 do
        G.C.UI_CHIPLICK[i] = math.min(math.max(((G.C.UI_CHIPS[i] * 0.5 + G.C.YELLOW[i] * 0.5) + 0.1) ^ 2, 0.1), 1)
        G.C.UI_MULTLICK[i] = math.min(math.max(((G.C.UI_MULT[i] * 0.5 + G.C.YELLOW[i] * 0.5) + 0.1) ^ 2, 0.1), 1)
    end

    G.ARGS.opponent_flame_handler = G.ARGS.opponent_flame_handler or {
        chips = {
            id = 'opponent_flame_chips',
            arg_tab = 'chip_flames',
            colour = G.C.UI_CHIPS,
            accent = G.C.UI_CHIPLICK
        },
        mult = {
            id = 'opponent_flame_mult',
            arg_tab = 'mult_flames',
            colour = G.C.UI_MULT,
            accent = G.C.UI_MULTLICK
        }
    }
    for k, v in pairs(G.ARGS.opponent_flame_handler) do
        if e.config.id == v.id then
            if not e.config.object:is(Sprite) or e.config.object.ID ~= v.ID then
                e.config.object:remove()
                e.config.object = Sprite(0, 0, 2.5, 2.5, G.ASSET_ATLAS["ui_1"], { x = 2, y = 0 })
                v.ID = e.config.object.ID
                G.ARGS[v.arg_tab] = {
                    intensity = 0,
                    real_intensity = 0,
                    intensity_vel = 0,
                    colour_1 = v.colour,
                    colour_2 = v.accent,
                    timer = G.TIMERS.REAL
                }
                e.config.object:set_alignment({
                    major = e.parent,
                    type = 'bmi',
                    offset = { x = 0, y = 0 },
                    xy_bond = 'Weak'
                })
                e.config.object:define_draw_steps({ {
                    shader = 'flame',
                    send = {
                        { name = 'time',            ref_table = G.ARGS[v.arg_tab],    ref_value = 'timer' },
                        { name = 'amount',          ref_table = G.ARGS[v.arg_tab],    ref_value = 'real_intensity' },
                        { name = 'image_details',   ref_table = e.config.object,      ref_value = 'image_dims' },
                        { name = 'texture_details', ref_table = e.config.object.RETS, ref_value = 'get_pos_pixel' },
                        { name = 'colour_1',        ref_table = G.ARGS[v.arg_tab],    ref_value = 'colour_1' },
                        { name = 'colour_2',        ref_table = G.ARGS[v.arg_tab],    ref_value = 'colour_2' },
                        { name = 'id',              val = e.config.object.ID },
                    }
                } })
                e.config.object:get_pos_pixel()
            end
            local _F = G.ARGS[v.arg_tab]
            local exptime = math.exp(-0.4 * G.real_dt)

            if G.ARGS.opponent_score_intensity.earned_score >= G.ARGS.opponent_score_intensity.required_score and G.ARGS.opponent_score_intensity.required_score > 0 then
                _F.intensity = ((G.pack_cards and not G.pack_cards.REMOVED) or (G.TAROT_INTERRUPT)) and 0 or
                    math.max(0., math.log(G.ARGS.opponent_score_intensity.earned_score, 5) - 2)
            else
                _F.intensity = 0
            end

            _F.timer = _F.timer + G.real_dt * (1 + _F.intensity * 0.2)
            if _F.intensity_vel < 0 then _F.intensity_vel = _F.intensity_vel * (1 - 10 * G.real_dt) end
            _F.intensity_vel = (1 - exptime) * (_F.intensity - _F.real_intensity) * G.real_dt * 25 +
                exptime * _F.intensity_vel
            _F.real_intensity = math.max(0, _F.real_intensity + _F.intensity_vel)
            _F.change = (_F.change or 0) * (1 - 4. * G.real_dt) +
                (4. * G.real_dt) * (_F.real_intensity < _F.intensity - 0.0 and 1 or 0) * _F.real_intensity
        end
    end
end

G.FUNCS.opponent_hand_mult_UI_set = function(e)
    local new_mult_text = number_format(G.GAME.opponent_current_round.current_hand.mult)
    if new_mult_text ~= G.GAME.opponent_current_round.current_hand.mult_text then
        G.GAME.opponent_current_round.current_hand.mult_text = new_mult_text
        e.config.object.scale = scale_number(G.GAME.opponent_current_round.current_hand.mult, 0.9, 1000)
        e.config.object:update_text()
        if not G.TAROT_INTERRUPT_PULSE then
            G.FUNCS.text_super_juice(e,
                math.max(0,
                    math.floor(math.log10(type(G.GAME.opponent_current_round.current_hand.mult) == 'number' and
                        G.GAME.opponent_current_round.current_hand.mult or 1))))
        end
    end
end

G.FUNCS.opponent_chip_UI_set = function(e)
    local new_chips_text = number_format(G.GAME.opponent_chips)
    if G.GAME.opponent_chips_text ~= new_chips_text then
        e.config.scale = math.min(0.8, scale_number(G.GAME.opponent_chips, 1.1))
        G.GAME.opponent_chips_text = new_chips_text
    end
end

G.FUNCS.opponent_update_round_won = function(e)
    local new_text = number_format(G.GAME.opponent_round_won)
    if G.GAME.opponent_round_won_text ~= new_text then
        e.config.scale = math.min(0.8, scale_number(G.GAME.opponent_round_won, 1.1))
        G.GAME.opponent_round_won_text = new_text

        local round_won = G.HUD:get_UIE_by_ID('opponent_round_won_text')

        --Popup text next to the chips in UI showing number of chips gained/lost
        round_won:juice_up()
        --Play a chip sound
        play_sound('chips2')
    end
end

G.FUNCS.update_round_won = function(e)
    local new_text = number_format(G.GAME.round_won)
    if G.GAME.round_won_text ~= new_text then
        e.config.scale = math.min(0.8, scale_number(G.GAME.round_won, 1.1))
        G.GAME.round_won_text = new_text

        local round_won = G.HUD:get_UIE_by_ID('round_won_text')

        --Popup text next to the chips in UI showing number of chips gained/lost
        round_won:juice_up()
        --Play a chip sound
        play_sound('chips2')
    end
end

opponent_ease_dollars = function(mod, instant)
    local function _mod(mod)
        local dollar_UI = G.HUD:get_UIE_by_ID('opponent_dollar_text_UI')
        mod = mod or 0
        local text = '+' .. localize('$')
        local col = G.C.MONEY
        if mod < 0 then
            text = '-' .. localize('$')
            col = G.C.RED
        else
            inc_career_stat('c_dollars_earned', mod)
        end
        --Ease from current chips to the new number of chips
        G.GAME.opponent_dollars = G.GAME.opponent_dollars + mod
        check_and_set_high_score('most_money', G.GAME.opponent_dollars)
        check_for_unlock({ type = 'money' })
        dollar_UI.config.object:update()
        G.HUD:recalculate()
        --Popup text next to the chips in UI showing number of chips gained/lost
        attention_text({
            text = text .. tostring(math.abs(mod)),
            scale = 0.8,
            hold = 0.7,
            cover = dollar_UI.parent,
            cover_colour = col,
            align = 'cm',
        })
        --Play a chip sound
        play_sound('coin1')
    end
    if instant then
        _mod(mod)
    else
        G.E_MANAGER:add_event(Event({
            trigger = 'immediate',
            func = function()
                _mod(mod)
                return true
            end
        }))
    end
end

G.FUNCS.opponent_discard_cards_from_highlighted = function(e, hook)
    stop_use()
    G.CONTROLLER.interrupt.focus = true
    G.CONTROLLER:save_cardarea_focus('hand')

    for k, v in ipairs(G.opponent_playing_cards) do
        v.ability.forced_selection = nil
    end

    if G.CONTROLLER.focused.target and G.CONTROLLER.focused.target.area == G.opponent_hand then
        G.card_area_focus_reset = {
            area =
                G.opponent_hand,
            rank = G.CONTROLLER.focused.target.rank
        }
    end
    local highlighted_count = math.min(#G.opponent_hand.highlighted,
        G.opponent_discard.config.card_limit - #G.opponent_play
        .cards)
    if highlighted_count > 0 then
        opponent_update_hand_text({ immediate = true, nopulse = true, delay = 0 },
            { mult = 0, chips = 0, level = '', handname = '' })
        table.sort(G.opponent_hand.highlighted, function(a, b) return a.T.x < b.T.x end)

        BALATRO_VS_CTX.interaction_context:handle_interaction(
            { before_pre_discard = true, full_hand = G.opponent_hand.highlighted }, {})
        for j = 1, #G.opponent_jokers.cards do
            G.opponent_jokers.cards[j]:opponent_calculate_joker({
                pre_discard = true,
                full_hand = G.opponent_hand.highlighted,
                hook =
                    hook
            })
        end
        local cards = {}
        local destroyed_cards = {}
        for i = 1, highlighted_count do
            G.opponent_hand.highlighted[i]:calculate_seal({ discard = true })
            local removed = false
            for j = 1, #G.opponent_jokers.cards do
                local eval = nil
                eval = G.opponent_jokers.cards[j]:opponent_calculate_joker({
                    discard = true,
                    other_card = G.opponent_hand.highlighted[i],
                    full_hand =
                        G.opponent_hand.highlighted
                })
                if eval then
                    if eval.remove then removed = true end
                    opponent_card_eval_status_text(G.opponent_jokers.cards[j], 'jokers', nil, 1, nil, eval)
                end
            end
            table.insert(cards, G.opponent_hand.highlighted[i])
            if removed then
                destroyed_cards[#destroyed_cards + 1] = G.opponent_hand.highlighted[i]
                if G.opponent_hand.highlighted[i].ability.name == 'Glass Card' then
                    G.opponent_hand.highlighted[i]:shatter()
                else
                    G.opponent_hand.highlighted[i]:start_dissolve()
                end
            else
                G.opponent_hand.highlighted[i].ability.discarded = true
                draw_card(G.opponent_hand, G.opponent_discard, i * 100 / highlighted_count, 'down', false,
                    G.opponent_hand.highlighted[i])
            end
        end

        if destroyed_cards[1] then
            for j = 1, #G.opponent_jokers.cards do
                opponent_eval_card(G.opponent_jokers.cards[j],
                    { cardarea = G.opponent_jokers, remove_playing_cards = true, removed = destroyed_cards })
            end
        end

        G.GAME.opponent_round_scores.cards_discarded.amt = G.GAME.opponent_round_scores.cards_discarded.amt + #cards
        if not hook then
            if G.GAME.modifiers.discard_cost then
                opponent_ease_dollars(-G.GAME.modifiers.discard_cost)
            end
            opponent_ease_discard(-1)
            G.GAME.opponent_current_round.discards_used = G.GAME.opponent_current_round.discards_used + 1
            G.E_MANAGER:add_event(Event({
                trigger = 'immediate',
                func = function()
                    G.FUNCS.opponent_draw_from_his_deck_to_his_hand()

                    G.E_MANAGER:add_event(Event({
                        trigger = 'after',
                        func = function()
                            G.FUNCS.game_manipulation_acknowledge_event()
                            return true
                        end
                    }))

                    return true
                end
            }))
        end
    end
end

function opponent_eval_card(card, context)
    context = context or {}
    local ret = {}

    if context.repetition_only then
        local seals = card:calculate_seal(context)
        if seals then
            ret.seals = seals
        end
        return ret
    end

    if context.cardarea == G.opponent_play then
        local chips = card:get_chip_bonus()
        if chips > 0 then
            ret.chips = chips
        end

        local mult = card:opponent_get_chip_mult()
        if mult > 0 then
            ret.mult = mult
        end

        local x_mult = card:get_chip_x_mult(context)
        if x_mult > 0 then
            ret.x_mult = x_mult
        end

        local p_dollars = card:get_p_dollars()
        if p_dollars > 0 then
            ret.p_dollars = p_dollars
        end

        local jokers = card:opponent_calculate_joker(context)
        if jokers then
            ret.jokers = jokers
        end

        local edition = card:get_edition(context)
        if edition then
            ret.edition = edition
        end
    end

    if context.cardarea == G.opponent_hand then
        local h_mult = card:get_chip_h_mult()
        if h_mult > 0 then
            ret.h_mult = h_mult
        end

        local h_x_mult = card:get_chip_h_x_mult()
        if h_x_mult > 0 then
            ret.x_mult = h_x_mult
        end

        local jokers = card:opponent_calculate_joker(context)
        if jokers then
            ret.jokers = jokers
        end
    end

    if context.cardarea == G.opponent_jokers or context.card == G.opponent_consumeables then
        local jokers = nil
        if context.edition then
            jokers = card:get_edition(context)
        elseif context.other_joker then
            jokers = context.other_joker:opponent_calculate_joker(context)
        else
            jokers = card:opponent_calculate_joker(context)
        end
        if jokers then
            ret.jokers = jokers
        end
    end

    return ret
end

function opponent_ease_discard(mod, instant, silent)
    local _mod = function(mod)
        if math.abs(math.max(G.GAME.opponent_current_round.discards_left, mod)) == 0 then return end
        local discard_UI = G.HUD:get_UIE_by_ID('opponent_discard_UI_count')
        mod = mod or 0
        mod = math.max(-G.GAME.opponent_current_round.discards_left, mod)
        local text = '+'
        local col = G.C.GREEN
        if mod < 0 then
            text = ''
            col = G.C.RED
        end
        --Ease from current chips to the new number of chips
        G.GAME.opponent_current_round.discards_left = G.GAME.opponent_current_round.discards_left + mod
        --Popup text next to the chips in UI showing number of chips gained/lost
        discard_UI.config.object:update()
        G.HUD:recalculate()
        attention_text({
            text = text .. mod,
            scale = 0.8,
            hold = 0.7,
            cover = discard_UI.parent,
            cover_colour = col,
            align = 'cm',
        })
        --Play a chip sound
        if not silent then play_sound('chips2') end
    end
    if instant then
        _mod(mod)
    else
        G.E_MANAGER:add_event(Event({
            trigger = 'immediate',
            func = function()
                _mod(mod)
                return true
            end
        }))
    end
end

G.FUNCS.draw_from_opponent_play_to_opponent_discard = function(e)
    local play_count = #G.opponent_play.cards
    local it = 1
    for k, v in ipairs(G.opponent_play.cards) do
        if (not v.shattered) and (not v.destroyed) then
            draw_card(G.opponent_play, G.opponent_discard, it * 100 / play_count, 'down', false, v)
            it = it + 1
        end
    end
end

G.FUNCS.network_send_to_opponent_new_cards_alignement = function(alignement, type)
    if network_send_to_opponent_new_cards_alignement then
        network_send_to_opponent_new_cards_alignement(alignement, type)
    end
end

G.FUNCS.network_player_sort_hand_suit = function()
    if network_player_sort_hand_suit then
        network_player_sort_hand_suit()
    end
end

G.FUNCS.network_player_sort_hand_value = function()
    if network_player_sort_hand_value then
        network_player_sort_hand_value()
    end
end

G.FUNCS.network_player_discarded_cards = function(discarded_cards)
    if network_player_discarded_cards then
        network_player_discarded_cards(discarded_cards)
    end
end

G.FUNCS.game_manipulation_acknowledge_event = function()
    if game_manipulation_acknowledge_event then
        game_manipulation_acknowledge_event()
    end
end

--TODO: Potentially useless function
G.FUNCS.network_send_new_card = function(type, card, location, stay_flipped)
    if network_send_new_card then
        local card_conf = {}
        card_conf.center = card.config.center and lume.serialize(card.config.center) or ''
        card_conf.card = card.config.card and lume.serialize(card.config.card) or ''
        card_conf.center_key = card.config.center_key or ''

        if type == 'joker' then
            card_conf.label = card.label
            card_conf.type_ = 'joker'
            card_conf.location = location or ''
            card_conf.stay_flipped = stay_flipped or false
            card_conf.edition = card.edition and lume.serialize(card.edition) or ''
        end

        if type == 'card' then
            card_conf.label = card.base.name
            card_conf.type_ = 'card'
        end

        if type == 'consumeable' then
            card_conf.label = card.config.center.name
            card_conf.type_ = 'consumeable'
        end

        network_send_new_card(card_conf)
    end
end

G.FUNCS.draw_from_opponnent_hand_to_opponent_discard = function(e)
    local hand_count = #G.opponent_hand.cards
    for i = 1, hand_count do
        draw_card(G.opponent_hand, G.opponent_discard, i * 100 / hand_count, 'down', nil, nil, 0.07)
    end
end

G.FUNCS.network_wait_for_opponent_action_on_end_shop = function()
    if not BALATRO_VS_CTX or not BALATRO_VS_CTX.network.is_live then
        G.FUNCS.toggle_shop()
        return
    end

    if network_wait_for_opponent_action_on_end_shop then
        G.shop:get_UIE_by_ID("next_round_button").states.visible = false
        G.shop:get_UIE_by_ID("reroll_shop_button").states.visible = false
        if G.shop_jokers.cards then remove_all(G.shop_jokers.cards) end
        G.shop_jokers.cards = {}

        if G.shop_vouchers.cards then remove_all(G.shop_vouchers.cards) end
        G.shop_vouchers.cards = {}

        if G.shop_booster.cards then remove_all(G.shop_booster.cards) end
        G.shop_booster.cards = {}

        BALATRO_VS_CTX.is_in_shop_and_ready = true
        network_wait_for_opponent_action_on_end_shop()
    end
end

G.FUNCS.network_wait_for_opponent_action_on_end_shop_after_events = function()
    if network_wait_for_opponent_action_on_end_shop_after_events then
        BALATRO_VS_CTX.is_in_shop_and_ready = false
        network_wait_for_opponent_action_on_end_shop_after_events()
    end
end


G.FUNCS.draw_from_opponent_discard_to_opponent_deck = function(e)
    G.E_MANAGER:add_event(Event({
        trigger = 'immediate',
        func = function()
            local discard_count = #G.opponent_discard.cards
            for i = 1, discard_count do --draw cards from deck
                draw_card(G.opponent_discard, G.opponent_deck, i * 100 / discard_count, 'up', nil, nil, 0.005, i % 2 == 0,
                    nil,
                    math.max((21 - i) / 20, 0.7))
            end
            return true
        end
    }))
end


G.FUNCS.network_player_use_consumeable_card = function(index, area_type, highlighted_cards_index, targets)
    if network_player_use_consumeable_card then
        network_player_use_consumeable_card(index, area_type, highlighted_cards_index, targets)
    end
end


function opponent_ease_hands_played(mod, instant)
    local _mod = function(mod)
        local hand_UI = G.HUD:get_UIE_by_ID('opponent_hand_UI_count')
        mod = mod or 0
        local text = '+'
        local col = G.C.GREEN
        if mod < 0 then
            text = ''
            col = G.C.RED
        end
        --Ease from current chips to the new number of chips
        G.GAME.opponent_current_round.hands_left = G.GAME.opponent_current_round.hands_left + mod
        hand_UI.config.object:update()
        G.HUD:recalculate()
        --Popup text next to the chips in UI showing number of chips gained/lost
        attention_text({
            text = text .. mod,
            scale = 0.8,
            hold = 0.7,
            cover = hand_UI.parent,
            cover_colour = col,
            align = 'cm',
        })
        --Play a chip sound
        play_sound('chips2')
    end
    if instant then
        _mod(mod)
    else
        G.E_MANAGER:add_event(Event({
            trigger = 'immediate',
            func = function()
                _mod(mod)
                return true
            end
        }))
    end
end

G.FUNCS.opponent_play_cards_from_highlighted = function()
    for k, v in ipairs(G.opponent_playing_cards) do
        v.ability.forced_selection = nil
    end

    opponent_ease_hands_played(-1)
    bvs_debug('opponent_play_cards_from_highlighted')
    G.E_MANAGER:add_event(Event({
        trigger = 'after',
        func = function()
            G.ACC = 0
            G.SPEEDFACTOR = 1
            G.FUNCS.opponent_evaluate_play()
            return true
        end
    }))

    G.E_MANAGER:add_event(Event({
        trigger = 'after',
        delay = 0.1,
        func = function()
            G.FUNCS.draw_from_opponent_play_to_opponent_discard()
            G.GAME.opponent_hands_played = G.GAME.opponent_hands_played + 1
            G.GAME.opponent_current_round.hands_played = G.GAME.opponent_current_round.hands_played + 1
            return true
        end
    }))
end

G.FUNCS.opponent_skip_booster = function(e)
    for i = 1, #G.opponent_jokers.cards do
        G.opponent_jokers.cards[i]:opponent_calculate_joker({ skipping_booster = true })
    end

    G.FUNCS.end_consumeable(e, 1, true)
end

G.FUNCS.network_player_skip_booster = function()
    if network_player_skip_booster then
        network_player_skip_booster()
    end
end

G.FUNCS.network_send_new_card_from_booster = function(index)
    if network_send_new_card_from_booster then
        network_send_new_card_from_booster(index)
    end
end


G.FUNCS.opponent_buy_from_shop = function(e)
    bvs_debug('opponent_buy_from_shop', e.config.id)

    local c1 = e.config.ref_table
    if c1 and c1:is(Card) then
        if is_online_card(c1.ability.name) then
            if not register_online_interaction(c1) then
                bvs_debug('Failed to register online interaction for card: ' .. c1.ability.name)
            end
        end

        G.E_MANAGER:add_event(Event({
            trigger = 'after',
            delay = 0.1,
            func = function()
                c1.area:remove_card(c1)
                c1:opponent_add_to_deck()
                if c1.children.price then c1.children.price:remove() end
                c1.children.price = nil
                if c1.children.buy_button then c1.children.buy_button:remove() end
                c1.children.buy_button = nil
                remove_nils(c1.children)
                if c1.ability.set == 'Default' or c1.ability.set == 'Enhanced' then
                    --G.playing_card = (G.playing_card and G.playing_card + 1) or 1
                    G.opponent_deck:emplace(c1)
                    --c1.playing_card = G.playing_card
                    opponent_playing_card_joker_effects({ c1 })
                    table.insert(G.opponent_playing_cards, c1)
                elseif e.config.id ~= 'buy_and_use' then
                    if c1.ability.consumeable then
                        G.opponent_consumeables:emplace(c1)
                    else
                        G.opponent_jokers:emplace(c1)
                    end
                    G.E_MANAGER:add_event(Event({
                        func = function()
                            c1:opponent_calculate_joker({ buying_card = true, card = c1 })
                            return true
                        end
                    }))
                end
                --Tallies for unlocks
                G.GAME.opponent_round_scores.cards_purchased.amt = G.GAME.opponent_round_scores.cards_purchased.amt + 1
                if c1.ability.set == 'Joker' then
                    G.GAME.opponent_current_round.jokers_purchased = G.GAME.opponent_current_round.jokers_purchased + 1
                end

                for i = 1, #G.opponent_jokers.cards do
                    G.opponent_jokers.cards[i]:opponent_calculate_joker({ buying_card = true, card = c1 })
                end

                -- if G.GAME.modifiers.inflation then
                --     G.GAME.inflation = G.GAME.inflation + 1
                --     G.E_MANAGER:add_event(Event({
                --         func = function()
                --             for k, v in pairs(G.I.CARD) do
                --                 if v.set_cost then v:set_cost() end
                --             end
                --             return true
                --         end
                --     }))
                -- end

                play_sound('card1')
                if c1.cost ~= 0 then
                    opponent_ease_dollars(-c1.cost)
                end

                if e.config.id == 'buy_and_use' then
                    G.FUNCS.use_card(e, true, nil, true)
                end

                G.E_MANAGER:add_event(Event({
                    trigger = 'after',
                    func = function()
                        G.FUNCS.game_manipulation_acknowledge_event()
                        return true
                    end
                }))
                return true
            end
        }))
    end
end

function opponent_create_playing_card(card_init, area, skip_materialize, silent, colours)
    card_init = card_init or {}
    card_init.front = card_init.front or pseudorandom_element(G.P_CARDS, opponent_pseudoseed('front'))
    card_init.center = card_init.center or G.P_CENTERS.c_base

    --G.playing_card = (G.playing_card and G.playing_card + 1) or 1
    local _area = area or G.opponent_hand
    local card = Card(_area.T.x, _area.T.y, G.CARD_W, G.CARD_H, card_init.front, card_init.center, {
        is_opponent = true
        --,playing_card = G.playing_card
    }
    )
    table.insert(G.opponent_playing_cards, card)
    --card.playing_card = G.playing_card

    if area then area:emplace(card) end
    if not skip_materialize then card:start_materialize(colours, silent) end

    return card
end

function opponent_find_joker(name, non_debuff)
    local jokers = {}
    if not G.opponent_jokers or not G.opponent_jokers.cards then return {} end
    for k, v in pairs(G.opponent_jokers.cards) do
        if v and type(v) == 'table' and v.ability.name == name and (non_debuff or not v.debuff) then
            table.insert(jokers, v)
        end
    end
    for k, v in pairs(G.opponent_consumeables.cards) do
        if v and type(v) == 'table' and v.ability.name == name and (non_debuff or not v.debuff) then
            table.insert(jokers, v)
        end
    end
    return jokers
end

function opponent_reset_idol_card()
    G.GAME.opponent_current_round.idol_card.rank = 'Ace'
    G.GAME.opponent_current_round.idol_card.suit = 'Spades'
    local valid_idol_cards = {}
    for k, v in ipairs(G.opponent_playing_cards) do
        if v.ability.effect ~= 'Stone Card' then
            valid_idol_cards[#valid_idol_cards + 1] = v
        end
    end
    if valid_idol_cards[1] then
        local idol_card = pseudorandom_element(valid_idol_cards,
            opponent_pseudoseed('idol' .. G.GAME.opponent_round_resets.ante))
        G.GAME.opponent_current_round.idol_card.rank = idol_card.base.value
        G.GAME.opponent_current_round.idol_card.suit = idol_card.base.suit
        G.GAME.opponent_current_round.idol_card.id = idol_card.base.id
    end
end

function opponent_reset_mail_rank()
    G.GAME.opponent_current_round.mail_card.rank = 'Ace'
    local valid_mail_cards = {}
    for k, v in ipairs(G.opponent_playing_cards) do
        if v.ability.effect ~= 'Stone Card' then
            valid_mail_cards[#valid_mail_cards + 1] = v
        end
    end
    if valid_mail_cards[1] then
        local mail_card = pseudorandom_element(valid_mail_cards, opponent_pseudoseed('mail' .. G.GAME.round_resets.ante))
        G.GAME.opponent_current_round.mail_card.rank = mail_card.base.value
        G.GAME.opponent_current_round.mail_card.id = mail_card.base.id
    end
end

function opponent_reset_ancient_card()
    local ancient_suits = {}
    for k, v in ipairs({ 'Spades', 'Hearts', 'Clubs', 'Diamonds' }) do
        if v ~= G.GAME.opponent_current_round.ancient_card.suit then ancient_suits[#ancient_suits + 1] = v end
    end
    local ancient_card = pseudorandom_element(ancient_suits, opponent_pseudoseed('anc' .. G.GAME.round_resets.ante))
    G.GAME.opponent_current_round.ancient_card.suit = ancient_card
end

function opponent_reset_castle_card()
    G.GAME.opponent_current_round.castle_card.suit = 'Spades'
    local valid_castle_cards = {}
    for k, v in ipairs(G.opponent_playing_cards) do
        if v.ability.effect ~= 'Stone Card' then
            valid_castle_cards[#valid_castle_cards + 1] = v
        end
    end
    if valid_castle_cards[1] then
        local castle_card = pseudorandom_element(valid_castle_cards,
            opponent_pseudoseed('cas' .. G.GAME.round_resets.ante))
        G.GAME.opponent_current_round.castle_card.suit = castle_card.base.suit
    end
end

G.FUNCS.opponent_reroll_shop = function(should_acknowledge)
    stop_use()
    G.CONTROLLER.locks.shop_reroll = true
    if G.CONTROLLER:save_cardarea_focus('shop_jokers') then G.CONTROLLER.interrupt.focus = true end
    if G.GAME.opponent_current_round.reroll_cost > 0 and should_acknowledge then
        opponent_ease_dollars(-G.GAME.opponent_current_round.reroll_cost)
    end
    G.E_MANAGER:add_event(Event({
        trigger = 'immediate',
        func = function()
            if should_acknowledge then
                local final_free = G.GAME.opponent_current_round.free_rerolls > 0
                G.GAME.opponent_current_round.free_rerolls = math.max(G.GAME.opponent_current_round.free_rerolls - 1, 0)
                G.GAME.opponent_round_scores.times_rerolled.amt = G.GAME.opponent_round_scores.times_rerolled.amt + 1

                opponent_calculate_reroll_cost(final_free)
            end

            for i = #G.shop_jokers.cards, 1, -1 do
                local c = G.shop_jokers:remove_card(G.shop_jokers.cards[i])
                c:remove()
                c = nil
            end

            play_sound('coin2')
            play_sound('other1')

            for i = 1, G.GAME.opponent_shop.joker_max - #G.shop_jokers.cards do
                local new_shop_card = opponent_create_card_for_shop(G.shop_jokers)
                G.shop_jokers:emplace(new_shop_card)
                new_shop_card:juice_up()
            end
            return true
        end
    }))
    if should_acknowledge then
        G.E_MANAGER:add_event(Event({
            trigger = 'after',
            delay = 0.3,
            func = function()
                G.E_MANAGER:add_event(Event({
                    func = function()
                        G.CONTROLLER.interrupt.focus = false
                        G.CONTROLLER.locks.shop_reroll = false
                        G.CONTROLLER:recall_cardarea_focus('shop_jokers')
                        for i = 1, #G.opponent_jokers.cards do
                            G.opponent_jokers.cards[i]:opponent_calculate_joker({ reroll_shop = true })
                        end

                        G.E_MANAGER:add_event(Event({
                            trigger = 'after',
                            delay = 3.0,
                            func = function()
                                G.FUNCS.game_manipulation_acknowledge_event() --Acknowledge the end of opponent reroll shop
                                return true
                            end
                        }))


                        return true
                    end
                }))
                return true
            end
        }))
    end
end

function opponent_calculate_reroll_cost(skip_increment)
    if G.GAME.opponent_current_round.free_rerolls < 0 then G.GAME.opponent_current_round.free_rerolls = 0 end
    if G.GAME.opponent_current_round.free_rerolls > 0 then
        G.GAME.opponent_current_round.reroll_cost = 0; return
    end
    G.GAME.opponent_current_round.reroll_cost_increase = G.GAME.opponent_current_round.reroll_cost_increase or 0
    if not skip_increment then
        G.GAME.opponent_current_round.reroll_cost_increase = G.GAME.opponent_current_round
            .reroll_cost_increase + 1
    end
    G.GAME.opponent_current_round.reroll_cost = (G.GAME.round_resets.temp_reroll_cost or G.GAME.round_resets.reroll_cost) +
        G.GAME.opponent_current_round.reroll_cost_increase
end

function opponent_create_card_for_shop(area)
    local forced_tag = nil

    G.GAME.spectral_rate = G.GAME.spectral_rate or 0
    local total_rate = G.GAME.joker_rate + G.GAME.tarot_rate + G.GAME.planet_rate + G.GAME.playing_card_rate +
        G.GAME.spectral_rate
    local polled_rate = pseudorandom(opponent_pseudoseed('cdt' .. G.GAME.opponent_round_resets.ante)) * total_rate
    local check_rate = 0
    for _, v in ipairs({
        { type = 'Joker',                                                                                                                        val = G.GAME.joker_rate },
        { type = 'Tarot',                                                                                                                        val = G.GAME.tarot_rate },
        { type = 'Planet',                                                                                                                       val = G.GAME.planet_rate },
        { type = (G.GAME.opponent_used_vouchers["v_illusion"] and pseudorandom(opponent_pseudoseed('illusion')) > 0.6) and 'Enhanced' or 'Base', val = G.GAME.playing_card_rate },
        { type = 'Spectral',                                                                                                                     val = G.GAME.spectral_rate },
    }) do
        if polled_rate > check_rate and polled_rate <= check_rate + v.val then
            local card = create_card(v.type, area, nil, nil, nil, nil, nil, 'sho', true)
            create_shop_card_ui(card, v.type, area)
            if (v.type == 'Base' or v.type == 'Enhanced') and G.GAME.opponent_used_vouchers["v_illusion"] and pseudorandom(opponent_pseudoseed('illusion')) > 0.8 then
                local edition_poll = pseudorandom(opponent_pseudoseed('illusion'))
                local edition = {}
                if edition_poll > 1 - 0.15 then
                    edition.polychrome = true
                elseif edition_poll > 0.5 then
                    edition.holo = true
                else
                    edition.foil = true
                end
                card:set_edition(edition)
            end
            return card
        end
        check_rate = check_rate + v.val
    end
end

function opponent_set_consumeable_usage(card)
    if card.config.center_key and card.ability.consumeable then
        if G.GAME.opponent_consumeable_usage[card.config.center_key] then
            G.GAME.opponent_consumeable_usage[card.config.center_key].count = G.GAME.opponent_consumeable_usage
                [card.config.center_key]
                .count + 1
        else
            G.GAME.opponent_consumeable_usage[card.config.center_key] = {
                count = 1,
                order = card.config.center.order,
                set = card
                    .ability.set
            }
        end
        G.GAME.opponent_consumeable_usage_total = G.GAME.opponent_consumeable_usage_total or
            { tarot = 0, planet = 0, spectral = 0, tarot_planet = 0, all = 0 }
        if card.config.center.set == 'Tarot' then
            G.GAME.opponent_consumeable_usage_total.tarot = G.GAME.opponent_consumeable_usage_total.tarot + 1
            G.GAME.opponent_consumeable_usage_total.tarot_planet = G.GAME.opponent_consumeable_usage_total.tarot_planet +
                1
        elseif card.config.center.set == 'Planet' then
            G.GAME.opponent_consumeable_usage_total.planet = G.GAME.opponent_consumeable_usage_total.planet + 1
            G.GAME.opponent_consumeable_usage_total.tarot_planet = G.GAME.opponent_consumeable_usage_total.tarot_planet +
                1
        elseif card.config.center.set == 'Spectral' then
            G.GAME.opponent_consumeable_usage_total.spectral = G.GAME.opponent_consumeable_usage_total.spectral + 1
        end

        G.GAME.opponent_consumeable_usage_total.all = G.GAME.opponent_consumeable_usage_total.all + 1

        if not card.config.center.discovered then
            discover_card(card)
        end

        if card.config.center.set == 'Tarot' or card.config.center.set == 'Planet' then
            G.E_MANAGER:add_event(Event({
                trigger = 'immediate',
                func = function()
                    G.E_MANAGER:add_event(Event({
                        trigger = 'immediate',
                        func = function()
                            G.GAME.opponent_last_tarot_planet = card.config.center_key
                            return true
                        end
                    }))
                    return true
                end
            }))
        end
    end
end

G.FUNCS.on_rematch = function()
    network_on_rematch()
end


G.FUNCS.is_rematch_active = function(e)
    if BALATRO_VS_CTX then
        if BALATRO_VS_CTX.network.is_rematch_requested then
            e.config.colour = G.C.UI.BACKGROUND_INACTIVE
            e.config.button = nil
        end
    end
end

G.FUNCS.can_go_to_main_menu = function(e)
    if BALATRO_VS_CTX then
        if BALATRO_VS_CTX.network.is_rematch_requested and BALATRO_VS_CTX.network.is_live then
            e.config.old_colour = e.config.colour
            e.config.colour = G.C.UI.BACKGROUND_INACTIVE
            e.config.button = nil
        end

        if BALATRO_VS_CTX.network.is_rematch_requested and not BALATRO_VS_CTX.network.is_live then
            e.config.colour = G.C.RED
            e.config.button = "go_to_menu"
        end
    end
end
