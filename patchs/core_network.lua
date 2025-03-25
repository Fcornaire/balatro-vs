function on_network_issue()
    play_sound('cancel')
    G.FUNCS.overlay_menu {
        definition =
            create_UIBox_generic_options({
                back_func = 'quit_and_return_to_vs_main_menu',
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
                                                { n = G.UIT.T, config = { text = "An error occured while communicating with the server...", scale = 0.5, colour = G.C.UI.TEXT_LIGHT, shadow = true } },
                                            }
                                        }
                                    end
                                }
                            }
                        }),
                }
            })
    }
end

function on_update()
    play_sound('whoosh', 1)
    G.FUNCS.overlay_menu {
        definition =
            create_UIBox_generic_options({
                contents = {
                    create_tabs(
                        {
                            scale = 1.5,
                            tabs =
                            {
                                {
                                    chosen = true,
                                    label = "Balatro versus",
                                    tab_definition_function = function()
                                        return
                                        {
                                            n = G.UIT.ROOT,
                                            config = { align = "cm", padding = 0.2, colour = G.C.BLACK, r = 0.1, emboss = 0.05, minh = 6, minw = 6 },
                                            nodes = {
                                                {
                                                    n = G.UIT.R,
                                                    config = { scale = 0.5, shadow = true },
                                                    nodes = {
                                                        { n = G.UIT.T, config = { text = "A new update is available !", scale = 0.85, colour = G.C.UI.TEXT_LIGHT, shadow = true } },
                                                    }
                                                },
                                                {
                                                    n = G.UIT.R,
                                                    config = { align = "cm", minw = 1.4, padding = 0.2 },
                                                    nodes = {
                                                    }
                                                },
                                                {
                                                    n = G.UIT.R,
                                                    config = { align = "cm", minw = 1.4, padding = 0.2 },
                                                    nodes = {
                                                        UIBox_button({
                                                            label = { localize('ph_click_confirm') },
                                                            button = 'updater_update_bvs',
                                                            minw = 2.5,
                                                            minh = 1.0,
                                                            colour = G.C.PURPLE,
                                                            scale = 2.5,
                                                            col = true,
                                                        })
                                                    }
                                                },
                                            }
                                        }
                                    end
                                }
                            }
                        }),
                }
            })
    }
end

function on_random_search()
    G.FUNCS.overlay_menu {
        definition =
            create_UIBox_generic_options({
                back_func = 'quit_and_return_to_vs_main_menu',
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
                                                { n = G.UIT.T, config = { text = "Waiting for an opponent...", scale = 0.5, colour = G.C.UI.TEXT_LIGHT, shadow = true } },
                                            }
                                        }
                                    end
                                }
                            }
                        }),
                }

            })

    }
end

on_rtt_updated = function(rtt)
    BALATRO_VS_CTX.network.rtt = rtt .. " ms"
    if G.OVERLAY_MENU then
        local ui = G.OVERLAY_MENU:get_UIE_by_ID('rtt_id')
        if ui then
            if rtt < 100 then
                ui.config.colour = G.C.GREEN
            elseif rtt < 200 then
                ui.config.colour = G.C.YELLOW
            else
                ui.config.colour = G.C.RED
            end
        end
    else
        if G.HUD then
            local ui = G.HUD:get_UIE_by_ID('rtt_id')
            if ui then
                if rtt < 100 then
                    ui.config.colour = G.C.GREEN
                elseif rtt < 200 then
                    ui.config.colour = G.C.YELLOW
                else
                    ui.config.colour = G.C.RED
                end
            end
        end
    end
end

function on_random_found()
    play_sound('whoosh', 1)
    G.FUNCS.overlay_menu {
        definition =
            create_UIBox_generic_options({
                back_func = 'quit_and_return_to_vs_main_menu',
                contents = {
                    create_tabs(
                        {
                            scale = 1.5,
                            tabs =
                            {
                                {
                                    chosen = true,
                                    label = "Random matchmaking",
                                    tab_definition_function = function()
                                        return
                                        {
                                            n = G.UIT.ROOT,
                                            config = { align = "cm", padding = 0.2, colour = G.C.BLACK, r = 0.1, emboss = 0.05, minh = 6, minw = 6 },
                                            nodes = {
                                                {
                                                    n = G.UIT.R,
                                                    config = { align = "cm", minw = 1.4, padding = 0.2 },
                                                    nodes = {
                                                        { n = G.UIT.T, config = { text = "Opponent found...", scale = 0.85, colour = G.C.UI.TEXT_LIGHT, shadow = true } },
                                                    }
                                                },
                                                {
                                                    n = G.UIT.R,
                                                    config = { align = "cm", minw = 1.4, padding = 0.2 },
                                                    nodes = {
                                                        { n = G.UIT.T, config = { ref_table = BALATRO_VS_CTX.network, ref_value = 'rtt', scale = 0.85, colour = G.C.WHITE, id = 'rtt_id', shadow = true } }
                                                    }
                                                },
                                                {
                                                    n = G.UIT.R,
                                                    config = { align = "cm", minw = 1.4, padding = 0.2 },
                                                    nodes = {
                                                        UIBox_button({
                                                            label = { localize('ph_click_confirm') },
                                                            button = 'network_confirm_versus_matchmaking',
                                                            minw = 2.5,
                                                            minh = 1.0,
                                                            colour = G.C.PURPLE,
                                                            scale = 2.5,
                                                            col = true,
                                                            func = 'has_not_confirmed_matchmaking'
                                                        })
                                                    }
                                                },
                                            }
                                        }
                                    end
                                }
                            }
                        }),
                }

            })

    }
end

function on_random_start(seed)
    print("Start match with seed: " .. seed)
    G.E_MANAGER:add_event(Event({
        trigger = 'immediate',
        no_delete = true,
        func = function()
            G.SETTINGS.GAMESPEED = 4
            G.SETTINGS.tutorial_complete = true
            G.FUNCS.network_wait_for_next_action()
            BALATRO_VS_CTX.network.is_live = true
            backup_progress()
            G.F_NO_SAVING = true
            G.FUNCS.start_run(e, { stake = 1, seed = seed, challenge = nil })
            return true
        end
    }))
end

function has_opponent_highlighted_cards()
    return network_has_opponent_highlithed_cards and network_has_opponent_highlithed_cards()
end

--Only relevant when maestro is present in opponent jokers area
--
--Only sending the player choice when we received opponent choice
function on_opponent_highlighted_card()
    if G.hand.locked and lume.any(G.opponent_jokers.cards, function(c)
            return c.config.center.key ==
                'j_online_discard_maestro'
        end) then
        BALATRO_VS_CTX.network.has_opponent_maestro_and_highlighted_cards = true
    end
end

function on_play_turn(cards)
    G.E_MANAGER:add_event(Event({
        trigger = 'after',
        delay = 0.2,
        func = function()
            --Highlight opponent cards
            for _, card_index in pairs(cards) do
                local card = G.opponent_hand.cards[card_index]
                if card then
                    card:click()
                    delay(0.5)
                end
            end

            delay(0.1)

            if #G.hand.highlighted > 0 then
                G.FUNCS.play_cards_from_highlighted()
            else
                G.E_MANAGER:add_event(Event({
                    trigger = 'after',
                    delay = 0.2,
                    func = function()
                        for i = 1, #G.opponent_hand.highlighted do
                            if G.opponent_hand.highlighted[i]:is_face() then inc_career_stat('c_face_cards_played', 1) end
                            G.opponent_hand.highlighted[i].base.times_played = G.opponent_hand.highlighted[i].base
                                .times_played + 1
                            G.opponent_hand.highlighted[i].ability.played_this_ante = true
                            G.GAME.opponent_round_scores.cards_played.amt = G.GAME.opponent_round_scores.cards_played
                                .amt + 1
                            draw_card(G.opponent_hand, G.opponent_play, i * 100 / #G.opponent_hand.highlighted, 'up', nil,
                                G.opponent_hand.highlighted[i])
                        end

                        return true
                    end
                }))

                G.E_MANAGER:add_event(Event({
                    trigger = 'after',
                    delay = 0.2,
                    func = function()
                        G.FUNCS.opponent_play_cards_from_highlighted()

                        G.E_MANAGER:add_event(Event({
                            trigger = 'after',
                            func = function()
                                G.FUNCS.opponent_draw_from_his_deck_to_his_hand()

                                G.E_MANAGER:add_event(Event({
                                    trigger = 'after',
                                    func = function()
                                        G.FUNCS.network_wait_for_next_action()
                                        G.FUNCS.game_manipulation_acknowledge_event()
                                        return true
                                    end
                                }))

                                return true
                            end
                        }))

                        G.E_MANAGER:add_event(Event({
                            trigger = 'after',
                            func = function()
                                G.E_MANAGER:add_event(Event({
                                    trigger = 'after',
                                    func = function()
                                        if G.GAME.opponent_current_round.hands_left == 0 then
                                            print("Opponent has no more hands left")
                                            G.STATE = G.STATES.NEW_ROUND
                                            G.STATE_COMPLETE = false
                                        end
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

            return true
        end
    }))
end

function on_opponent_discarded_cards(cards)
    G.E_MANAGER:add_event(Event({
        trigger = 'after',
        func = function()
            --Highlight opponent cards
            for _, card_index in pairs(cards) do
                local card = G.opponent_hand.cards[card_index]
                if card then
                    card:click()
                end
            end

            --delay(1.5) --Delay when card are not flipped ?

            G.FUNCS.opponent_discard_cards_from_highlighted()

            return true
        end
    }))
end

local function reallign_cards(original, new_alignement)
    local rearranged_table = {}

    for new_index, original_index in ipairs(new_alignement) do
        rearranged_table[new_index] = original[original_index]
    end

    for i, card in ipairs(rearranged_table) do
        original[i] = card
    end
end

function get_opponent_hand_length()
    return #G.opponent_hand.cards
end

function get_opponent_jokers_length()
    return #G.opponent_jokers.cards
end

function get_opponent_consumeables_length()
    return #G.opponent_consumeables.cards
end

function on_opponent_new_card_alignement(new_alignement, type)
    G.E_MANAGER:add_event(Event({
        trigger = 'immediate',
        func = function()
            local card_types = {
                hand = G.opponent_hand.cards,
                consumeables = G.opponent_consumeables.cards, --Wait, you can't move consumeables...
                jokers = G.opponent_jokers.cards
            }

            if card_types[type] then
                reallign_cards(card_types[type], new_alignement)
            end

            return true
        end
    }))
end

function on_opponent_sort_hand_suit()
    G.E_MANAGER:add_event(Event({
        trigger = 'immediate',
        func = function()
            G.opponent_hand:sort('suit desc')
            play_sound('paper1')
            return true
        end
    }))
end

function on_opponent_sort_hand_value()
    G.E_MANAGER:add_event(Event({
        trigger = 'immediate',
        func = function()
            G.opponent_hand:sort('desc')
            play_sound('paper1')
            return true
        end
    }))
end

function on_opponent_new_card(card_conf)
    print("Opponent placed a card with label: " .. card_conf.label)
    local center = lume.deserialize(card_conf.center)
    local card = lume.deserialize(card_conf.card)
    local center_key = card_conf.center_key or ''

    if card_conf.type_ == 'joker' then
        local joker_card = Card(G.opponent_jokers.T.x,
            G.opponent_jokers.T.y, G.CARD_W, G.CARD_H, card, center,
            { bypass_discovery_center = true, bypass_discovery_ui = true, is_opponent = true })
        joker_card.config.center_key = center_key

        if card_conf.location and card_conf.location ~= '' then
            joker_card.location = card_conf.location
        end

        joker_card.edition = lume.deserialize(card_conf.edition) or nil
        joker_card:start_materialize()
        G.opponent_jokers:emplace(joker_card)
    end

    if card_conf.type_ == 'card' then
        local card = Card(G.opponent_play.T.x,
            G.opponent_play.T.y, G.CARD_W, G.CARD_H, card, center,
            { bypass_discovery_center = true, bypass_discovery_ui = true, is_opponent = true })
        card.config.center_key = center_key

        card:start_materialize()
        G.opponent_deck:emplace(card)
    end

    if card_conf.type_ == 'consumeable' then
        local card = Card(G.opponent_play.T.x,
            G.opponent_play.T.y, G.CARD_W, G.CARD_H, card, center,
            { bypass_discovery_center = true, bypass_discovery_ui = true, is_opponent = true })
        card.config.center_key = center_key

        card:start_materialize()
        G.opponent_consumeables:emplace(card)
    end

    play_sound('card1', 0.8, 0.6)
    play_sound('generic1')
end

function on_end_shop()
    -- Re shuffle both deck because of potential new cards
    delay(0.1);
    ease_ante(1)

    G.deck:shuffle('nr' .. G.GAME.round_resets.ante)
    G.deck:hard_set_T()
    G.opponent_deck:shuffle('nr' .. G.GAME.round_resets.ante)
    G.opponent_deck:hard_set_T()

    --Reset chip
    G.GAME.chips = 0
    G.GAME.opponent_chips = 0

    BALATRO_VS_CTX.is_opponent_first_reroll_shop = true --New round start, reset this for next shop
    G.GAME.opponent_pseudorandom = {}                   --Reset opponent random for next round (this have no repercussion ?)

    G.E_MANAGER:add_event(Event({
        trigger = 'immediate',
        func = function()
            on_cash_out(G.GAME.dollars, true) --Re send $
            return true
        end
    }))

    G.FUNCS.toggle_shop()
end

--To determine what is the next action after processing the past events
function on_processed_remaining_events()
    G.E_MANAGER:add_event(Event({
        trigger = 'immediate',
        func = function()
            --We have processed remaining events on shop, we need now to wait for opponent finishing to process his events
            if G.STATE == G.STATES.SHOP then
                G.FUNCS.network_wait_for_opponent_action_on_end_shop_after_events()
            end

            --Booster pack here mean we received opponent shop choice, we can safely tell the opponent we are ready
            if lume.any({
                        G.STATES.STANDARD_PACK,
                        G.STATES.TAROT_PACK,
                        G.STATES.PLANET_PACK,
                        G.STATES.SPECTRAL_PACK,
                        G.STATES.BUFFOON_PACK },
                    function(states) return G.STATE == states end)
                or G.STATE == nil --For some reason, the previous state can be nil but it should not matter that much
            then
                G.STATE = G.GAME.PACK_INTERRUPT
                G.FUNCS.network_wait_for_opponent_action_on_end_shop_after_events()
            end

            return true
        end
    }))
end

function on_opponent_use_consumeable_card(index, is_consumeable, highlighted_cards_index, targets)
    print("Opponent used a consumeable card with index: " .. index)

    G.E_MANAGER:add_event(Event({
        trigger = 'immediate',
        func = function()
            if is_consumeable then
                --Highlight opponent cards
                for _, card_index in pairs(highlighted_cards_index) do
                    local card = G.opponent_hand.cards[card_index]
                    if card then
                        card:click()
                    end
                end

                delay(0.1)

                local consumeable_card = G.opponent_consumeables.cards[index] --From raw consumeable area

                if consumeable_card then
                    --Handle online cards
                    if is_online_card(consumeable_card.ability.name) then
                        consumeable_card.ability.targets = {}
                        for i, card_conf in pairs(targets) do
                            for _, joker in pairs(G.jokers.cards) do
                                if joker.ability.name == card_conf.label and joker.balatro_vs_center_id == card_conf.versus_center_id then
                                    table.insert(consumeable_card.ability.targets, joker)
                                    break
                                end
                            end
                        end
                    end

                    G.E_MANAGER:add_event(Event({
                        trigger = 'immediate',
                        func = function()
                            G.FUNCS.use_card({ config = { ref_table = consumeable_card } }, nil, nil, true)

                            G.E_MANAGER:add_event(Event({
                                trigger = 'after',
                                func = function()
                                    --Unhighlight opponent cards
                                    G.opponent_hand:unhighlight_all()

                                    G.E_MANAGER:add_event(Event({
                                        trigger = 'after',
                                        func = function()
                                            G.FUNCS.game_manipulation_acknowledge_event() --Acknowledge the end of opponent use card
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
            else
                G.E_MANAGER:add_event(Event({
                    trigger = 'immediate',
                    func = function()
                        --Flip opponent cards
                        for _, card_index in pairs(highlighted_cards_index) do
                            local card = G.opponent_hand.cards[card_index]
                            if card then
                                card:flip()
                            end
                        end

                        G.E_MANAGER:add_event(Event({
                            trigger = 'after',
                            delay = 2.0,
                            func = function()
                                --Highlight opponent cards
                                for _, card_index in pairs(highlighted_cards_index) do
                                    local card = G.opponent_hand.cards[card_index]
                                    if card then
                                        card:click()
                                    end
                                end

                                G.E_MANAGER:add_event(Event({
                                    trigger = 'after',
                                    delay = 3.0,
                                    func = function()
                                        local card = G.pack_cards.cards[index] --Todo: fix potential crash here
                                        G.FUNCS.use_card({ config = { ref_table = card } }, nil, nil, true)

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

            return true
        end
    }))
end

G.FUNCS.network_player_use_voucher_card = function(card)
    if network_player_use_voucher_card then
        local card_conf = {}
        card_conf.center = card.config.center and lume.serialize(card.config.center) or ''
        card_conf.card = card.config.card and lume.serialize(card.config.card) or ''
        card_conf.center_key = card.config.center_key

        card_conf.label = card.ability.name
        card_conf.type_ = 'voucher'

        network_player_use_voucher_card(card_conf)
    end
end

function on_opponent_use_voucher_card(card_conf)
    print("Opponent used a voucher card with label: " .. card_conf.label)
    local center = lume.deserialize(card_conf.center)
    local card = lume.deserialize(card_conf.card)
    local center_key = card_conf.center_key

    local voucher_card = Card(G.opponent_play.T.x,
        G.opponent_play.T.y, G.CARD_W, G.CARD_H, card, center,
        { bypass_discovery_center = true, bypass_discovery_ui = true, is_opponent = true })

    voucher_card:start_materialize()
    voucher_card.area = G.opponent_play
    voucher_card.config.center_key = center_key
    voucher_card.is_opponent = true

    G.FUNCS.use_card({ config = { ref_table = voucher_card } }, nil, nil, true)

    G.E_MANAGER:add_event(Event({
        trigger = 'after',
        func = function()
            G.FUNCS.game_manipulation_acknowledge_event() --Acknowledge the end of opponent used voucher card
            return true
        end
    }))
end

function get_current_hands_left()
    return G.GAME.current_round.hands_left
end

G.FUNCS.network_send_open_booster = function(card, shop_jokers_cards)
    if network_send_open_booster then
        local card_conf = {}
        card_conf.center = card.config.center and lume.serialize(card.config.center) or ''
        card_conf.card = card.config.card and lume.serialize(card.config.card) or ''
        card_conf.ability = card.ability and lume.serialize(card.ability) or ''
        card_conf.center_key = card.config.center_key

        card_conf.label = card.ability.name
        card_conf.type_ = 'booster'

        -- also send the shop jokers cards
        local shop_jokers_cards_conf = {}
        for _, shop_joker in pairs(shop_jokers_cards) do
            if not is_online_card(shop_joker.config.center.key) then --Unless we have in the future online booster cards
                local shop_joker_conf = {}
                shop_joker_conf.center = shop_joker.config.center and lume.serialize(shop_joker.config.center) or ''
                shop_joker_conf.card = shop_joker.config.card and lume.serialize(shop_joker.config.card) or ''
                shop_joker_conf.ability = shop_joker.ability and lume.serialize(shop_joker.ability) or ''
                shop_joker_conf.center_key = shop_joker.config.center_key
                shop_joker_conf.label = shop_joker.ability.name
                shop_joker_conf.type_ = string.lower(shop_joker.config.center.set)
                table.insert(shop_jokers_cards_conf, shop_joker_conf)
            end
        end

        network_send_open_booster(card_conf, shop_jokers_cards_conf)
    end
end


function on_opponent_open_booster(card_conf, shop_jokers_cards)
    print("Opponent opened a booster card with label: " .. card_conf.label)

    G.E_MANAGER:add_event(Event({
        trigger = 'immediate',
        func = function()
            local center = lume.deserialize(card_conf.center)
            local card = lume.deserialize(card_conf.card)

            local booster = Card(G.opponent_play.T.x,
                G.opponent_play.T.y, G.CARD_W, G.CARD_H, card, center,
                { bypass_discovery_center = true, bypass_discovery_ui = true, is_opponent = true })
            booster.config.center_key = card_conf.center_key

            booster:start_materialize()
            booster.ability = lume.deserialize(card_conf.ability)
            booster.area = G.opponent_play
            booster.is_opponent = true

            --deserialize and emplace the shop jokers cards
            if G.shop_jokers.cards then remove_all(G.shop_jokers.cards) end
            for _, shop_joker in pairs(shop_jokers_cards) do
                local center = lume.deserialize(shop_joker.center)
                local card = lume.deserialize(shop_joker.card)

                local shop_joker_card = Card(G.shop_jokers.T.x,
                    G.shop_jokers.T.y, G.CARD_W, G.CARD_H, card, center,
                    { bypass_discovery_center = true, bypass_discovery_ui = true, is_opponent = true })
                shop_joker_card.config.center_key = shop_joker.center_key

                shop_joker_card:start_materialize()
                shop_joker_card.ability = lume.deserialize(shop_joker.ability)
                shop_joker_card.area = G.shop_jokers
                G.shop_jokers:emplace(shop_joker_card)
            end

            booster:opponent_open()

            return true
        end
    }))
end

function on_highlighted_booster_card(highlighted_card_index)
    print("Highlighting booster cards")

    G.E_MANAGER:add_event(Event({
        trigger = 'immediate',
        func = function()
            if #highlighted_card_index == 0 then
                print("No booster cards to highlight,skipping")
                G.FUNCS.opponent_skip_booster(nil)
                return true
            end

            --Highlight opponent card , TODO: can we have multiple cards highlighted? ?
            local card = G.pack_cards.cards[highlighted_card_index[1]]
            if card then
                -- card:click()
                -- delay(0.1)

                G.FUNCS.use_card({ config = { ref_table = card } }, nil, nil, true)
            end


            return true
        end
    }))
end

function on_update_message(message)
    G.E_MANAGER:add_event(Event({
        trigger = 'immediate',
        func = function()
            if message and message ~= '' then
                if G.GAME.boss_warning_text then
                    G.GAME.boss_warning_text:remove()
                    G.GAME.boss_warning_text = nil
                end

                G.GAME.boss_warning_text = UIBox {
                    definition =
                    { n = G.UIT.ROOT, config = { align = 'cm', colour = G.C.CLEAR, padding = 0.2 }, nodes = {
                        { n = G.UIT.R, config = { align = 'cm', maxw = 1 }, nodes = {
                            { n = G.UIT.O, config = { object = DynaText({ scale = 1.0, string = message, maxw = 9, colours = { G.C.WHITE }, float = true, shadow = true, silent = true, pop_in = 0, pop_in_rate = 6 }) } },
                        } },
                    } },
                    config = {
                        align = 'cm',
                        offset = { x = 0, y = -2.1 },
                        major = G.opponent_play,
                    }
                }
                G.GAME.boss_warning_text.attention_text = true
                G.GAME.boss_warning_text.states.collide.can = false
                play_sound('chips1', math.random() * 0.1 + 0.55, 0.12)
            else
                if G.GAME.boss_warning_text then
                    G.GAME.boss_warning_text:remove()
                    G.GAME.boss_warning_text = nil
                end
            end
            return true
        end
    }))
end

function update_message_instant(message, scale)
    if message and message ~= '' then
        if G.GAME.boss_warning_text then
            G.GAME.boss_warning_text:remove()
            G.GAME.boss_warning_text = nil
        end

        G.GAME.boss_warning_text = UIBox {
            definition =
            { n = G.UIT.ROOT, config = { align = 'cm', colour = G.C.CLEAR, padding = 0.2 }, nodes = {
                { n = G.UIT.R, config = { align = 'cm', maxw = 1 }, nodes = {
                    { n = G.UIT.O, config = { object = DynaText({ scale = scale, string = message, maxw = 9, colours = { G.C.WHITE }, float = true, shadow = true, silent = true, }) } },
                } },
            } },
            config = {
                align = 'cm',
                offset = { x = 0, y = -2.1 },
                major = G.opponent_play,
            }
        }
        G.GAME.boss_warning_text.attention_text = true
        G.GAME.boss_warning_text.states.collide.can = false
    else
        if G.GAME.boss_warning_text then
            G.GAME.boss_warning_text:remove()
            G.GAME.boss_warning_text = nil
        end
    end
end

function on_reroll_shop()
    if network_send_reroll_shop then
        network_send_reroll_shop()
    end
end

function on_opponent_reroll_shop()
    --Opponent reroll his shop, we only care about joker calculation

    G.E_MANAGER:add_event(Event({
        trigger = 'immediate',
        func = function()
            --Because of opponent_pseudoseed saving player seed gen, and the opponent already rerolled his shop (the first time)
            --We do a first reroll to consume the first saved seed saved by the opponent_pseudoseed
            if BALATRO_VS_CTX.is_opponent_first_reroll_shop then
                BALATRO_VS_CTX.is_opponent_first_reroll_shop = false
                G.FUNCS.opponent_reroll_shop(false) --false because no acknowledge needed on an empty reroll
            end

            G.FUNCS.opponent_reroll_shop(true)
            return true
        end
    }))
end

function on_send_bought_card(card, type, id)
    if network_send_bought_card then
        local card_conf = {}
        card_conf.center = is_online_card(card.config.center.key) and
            lume.serialize(get_center_for_online_card_by_name(card.config.center.name))
            or (card.config.center and lume.serialize(card.config.center) or '')
        card_conf.card = card.config.card and lume.serialize(card.config.card) or ''
        card_conf.center_key = is_online_card(card.config.center.key) and card.config.center.key or
            card.config.center_key

        if type == 'joker' then
            card_conf.label = card.label
            card_conf.type_ = 'joker'
            card_conf.location = ''
            card_conf.stay_flipped = false
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

        network_send_bought_card(card_conf, id)
    end
end

function on_opponent_bought_card(card_conf, id) --TODO: No need to send the whole card as the reroll shop should be showing the opponent bought card
    if G.shop_jokers.cards then remove_all(G.shop_jokers.cards) end
    G.shop_jokers.cards = {}

    local center = lume.deserialize(card_conf.center)
    local card = lume.deserialize(card_conf.card)
    local center_key = card_conf.center_key

    local card = Card(G.shop_jokers.T.x,
        G.shop_jokers.T.y, G.CARD_W, G.CARD_H, card, center,
        { bypass_discovery_center = true, bypass_discovery_ui = true, is_opponent = true })
    card.config.center_key = center_key
    card.edition = lume.deserialize(card_conf.edition) or nil
    card:start_materialize()
    card.states.click.can = false
    G.shop_jokers:emplace(card)

    create_shop_card_ui(card, card.type, G.shop_jokers)

    delay(2)

    G.FUNCS.opponent_buy_from_shop({
        config = {
            ref_table = card,
            id = id
        }
    })
end

function on_sell_card(card)
    local index = card:get_index_from_area()
    if index > 0 then
        local is_consumeable = card.area == G.consumeables
        if network_send_sell_card then
            network_send_sell_card(index, is_consumeable)
        end
    end
end

function on_opponent_sell_card(index, is_consumeable)
    print("Opponent sold a card with index: " .. index)
    G.E_MANAGER:add_event(Event({
        trigger = 'immediate',
        func = function()
            local card
            if is_consumeable then
                card = G.opponent_consumeables.cards[index]
            else
                card = G.opponent_jokers.cards[index]
            end

            if card then
                G.E_MANAGER:add_event(Event({
                    trigger = 'after',
                    delay = 0.5,
                    func = function()
                        card:sell_card(true)

                        --Trigger joker calculation after selling
                        for i = 1, #G.opponent_jokers.cards do
                            if G.opponent_jokers.cards[i] ~= card then
                                G.opponent_jokers.cards[i]:opponent_calculate_joker({ selling_card = true, card = card })
                            end
                        end

                        G.E_MANAGER:add_event(Event({
                            trigger = 'after',
                            delay = 0.5,
                            func = function()
                                G.FUNCS.game_manipulation_acknowledge_event() --Acknowledge the end of opponent sell card
                                return true
                            end
                        }))
                        return true
                    end
                }))
            end

            return true
        end
    }))
end

function on_cash_out(new_dollars, is_ending_shop)
    if network_send_cash_out then
        network_send_cash_out(new_dollars, is_ending_shop)
    end
end

function on_opponent_cash_out(new_dollars)
    G.E_MANAGER:add_event(Event({
        trigger = 'immediate',
        func = function()
            local to_ease = new_dollars >= G.GAME.opponent_dollars and new_dollars - G.GAME.opponent_dollars or
                G.GAME.opponent_dollars - new_dollars

            if to_ease ~= 0 then
                opponent_ease_dollars(to_ease, true)
            end
            return true
        end
    }))
end

function on_opponent_disconnected_from_found()
    print("Opponent disconnected before accepting the match")
    BALATRO_VS_CTX.network.has_confirmed_matchmaking = false
    play_sound('cancel', 1)
    G.FUNCS.overlay_menu {
        definition =
            create_UIBox_generic_options({
                back_func = 'quit_and_return_to_vs_main_menu',
                contents = {
                    create_tabs(
                        {
                            scale = 1.5,
                            tabs =
                            {
                                {
                                    chosen = true,
                                    label = "Random matchmaking",
                                    tab_definition_function = function()
                                        return
                                        {
                                            n = G.UIT.ROOT,
                                            config = { align = "cm", padding = 0.2, colour = G.C.BLACK, r = 0.1, emboss = 0.05, minh = 6, minw = 6 },
                                            nodes = {
                                                {
                                                    n = G.UIT.R,
                                                    config = { align = "cm", minw = 1.4, padding = 0.2 },
                                                    nodes = {
                                                        { n = G.UIT.T, config = { text = "Opponent declined", scale = 0.85, colour = G.C.UI.TEXT_LIGHT, shadow = true } },
                                                    }
                                                },
                                                {
                                                    n = G.UIT.R,
                                                    config = { align = "cm", minw = 1.4, padding = 0.2 },
                                                    nodes = {
                                                        UIBox_button({
                                                            label = { localize('b_versus_rematch') },
                                                            button = 'versus_matchmaking_start',
                                                            minw = 2.5,
                                                            minh = 1.0,
                                                            colour = G.C.PURPLE,
                                                            scale = 2.5,
                                                            col = true,
                                                        })
                                                    }
                                                },
                                            }
                                        }
                                    end
                                }
                            }
                        }),
                }

            })

    }
end

function on_opponent_disconnected_in_game()
    BALATRO_VS_CTX.network.is_live = false
    BALATRO_VS_CTX.network.has_confirmed_matchmaking = false
    BALATRO_VS_CTX.network.should_reset = true
    BALATRO_VS_CTX.timer:stop()
    BALATRO_VS_CTX.timer = nil
    G.opp_ext_code = ''

    play_sound('negative', 1)
    G.SETTINGS.paused = true
    local eased_purple = copy_table(G.C.PURPLE)
    eased_purple[4] = 0
    ease_value(eased_purple, 4, 0.8, nil, nil, true)

    G.FUNCS.overlay_menu {
        definition = create_UIBox_generic_options({ padding = 0.5, bg_colour = eased_purple, colour = G.C.BLACK, outline_colour = G.C.EDITION, no_back = true, no_esc = true, contents = {
            { n = G.UIT.R, config = { align = "cm" }, nodes = {
                { n = G.UIT.O, config = { object = DynaText({ string = { localize('b_versus_opponent_disconnected') }, colours = { G.C.EDITION }, shadow = true, float = true, spacing = 10, rotate = true, scale = 1.5, pop_in = 0.4, maxw = 6.5 }) } },
            } },
            { n = G.UIT.R, config = { align = "cm", padding = 0.15 }, nodes = {
                { n = G.UIT.C, config = { align = "cm" }, nodes = {
                    { n = G.UIT.R, config = { align = "cm", padding = 0.08 }, nodes = {
                        UIBox_button({ button = 'go_to_menu', label = { localize('b_main_menu') }, minw = 2.5, maxw = 2.5, minh = 1, focus_args = { nav = 'wide' } }) } },
                } }
            } }
        },
            config = { no_esc = true } })
    }
end
